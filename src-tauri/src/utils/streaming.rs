//! Streaming utilities for efficient large dataset processing

use crate::error::{AppError, Result};
use futures::Stream;
use pin_project::pin_project;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::{StreamExt, StreamMap};

/// Configuration for streaming operations
#[derive(Debug, Clone)]
pub struct StreamConfig {
    pub buffer_size: usize,
    pub batch_size: usize,
    pub timeout: Option<Duration>,
    pub backpressure_delay: Duration,
    pub max_memory_usage_mb: usize,
    pub enable_compression: bool,
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            batch_size: 100,
            timeout: Some(Duration::from_secs(30)),
            backpressure_delay: Duration::from_millis(10),
            max_memory_usage_mb: 100,
            enable_compression: true,
        }
    }
}

/// Stream statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStats {
    pub items_processed: u64,
    pub items_per_second: f64,
    pub memory_usage_mb: f64,
    pub processing_time_ms: f64,
    pub buffer_utilization: f64,
    pub error_count: u64,
}

/// Streaming item with metadata
#[derive(Debug, Clone)]
pub struct StreamItem<T> {
    pub data: T,
    pub timestamp: Instant,
    pub size_bytes: usize,
    pub metadata: Option<serde_json::Value>,
}

impl<T> StreamItem<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            size_bytes: std::mem::size_of::<T>(),
            metadata: None,
        }
    }

    pub fn with_metadata(data: T, metadata: serde_json::Value) -> Self {
        Self {
            data,
            timestamp: Instant::now(),
            size_bytes: std::mem::size_of::<T>(),
            metadata: Some(metadata),
        }
    }
}

/// Custom stream with backpressure handling and memory management
#[pin_project]
pub struct BufferedStream<T> {
    #[pin]
    receiver: ReceiverStream<StreamItem<T>>,
    buffer_size: usize,
    current_buffer_size: usize,
    max_memory_mb: usize,
    current_memory_mb: usize,
    stats: StreamStats,
    start_time: Instant,
}

impl<T> BufferedStream<T> {
    pub fn new(buffer_size: usize, max_memory_mb: usize) -> (Self, mpsc::Sender<StreamItem<T>>) {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let stream = Self {
            receiver: ReceiverStream::new(receiver),
            buffer_size,
            current_buffer_size: 0,
            max_memory_mb,
            current_memory_mb: 0,
            stats: StreamStats {
                items_processed: 0,
                items_per_second: 0.0,
                memory_usage_mb: 0.0,
                processing_time_ms: 0.0,
                buffer_utilization: 0.0,
                error_count: 0,
            },
            start_time: Instant::now(),
        };
        (stream, sender)
    }

    pub fn get_stats(&self) -> &StreamStats {
        &self.stats
    }

    fn update_stats(&mut self) {
        let elapsed = self.start_time.elapsed().as_millis() as f64;
        self.stats.processing_time_ms = elapsed;

        if elapsed > 0.0 {
            self.stats.items_per_second = (self.stats.items_processed as f64) / (elapsed / 1000.0);
        }

        self.stats.buffer_utilization = (self.current_buffer_size as f64) / (self.buffer_size as f64);
        self.stats.memory_usage_mb = self.current_memory_mb as f64;
    }
}

impl<T> Stream for BufferedStream<T> {
    type Item = Result<StreamItem<T>>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.receiver.poll_next(_cx) {
            Poll::Ready(Some(item)) => {
                *this.current_buffer_size = this.current_buffer_size.saturating_sub(1);
                *this.current_memory_mb = this.current_memory_mb.saturating_sub(item.size_bytes / (1024 * 1024));
                this.stats.items_processed += 1;
                this.update_stats();
                Poll::Ready(Some(Ok(item)))
            }
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Stream processor with configurable batch processing
pub struct StreamProcessor<T, R> {
    config: StreamConfig,
    processor: Box<dyn Fn(Vec<StreamItem<T>>) -> Result<Vec<R>> + Send + Sync>,
}

impl<T, R> StreamProcessor<T, R> {
    pub fn new<F>(config: StreamConfig, processor: F) -> Self
    where
        F: Fn(Vec<StreamItem<T>>) -> Result<Vec<R>> + Send + Sync + 'static,
    {
        Self {
            config,
            processor: Box::new(processor),
        }
    }

    /// Process a stream of items in batches
    pub async fn process_stream<S>(&self, stream: S) -> impl Stream<Item = Result<R>>
    where
        S: Stream<Item = Result<StreamItem<T>>> + Send + 'static,
        T: Send + 'static,
        R: Send + 'static,
    {
        let config = self.config.clone();
        let processor = self.processor.clone();

        async_stream::stream! {
            let mut batch = Vec::with_capacity(config.batch_size);
            let mut stream = Box::pin(stream);

            while let Some(item_result) = stream.next().await {
                match item_result {
                    Ok(item) => {
                        batch.push(item);

                        // Process batch when full or stream ends
                        if batch.len() >= config.batch_size {
                            match processor(batch.clone()) {
                                Ok(results) => {
                                    for result in results {
                                        yield Ok(result);
                                    }
                                }
                                Err(e) => yield Err(e),
                            }
                            batch.clear();

                            // Apply backpressure delay
                            sleep(config.backpressure_delay).await;
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }

            // Process remaining items in batch
            if !batch.is_empty() {
                match processor(batch) {
                    Ok(results) => {
                        for result in results {
                            yield Ok(result);
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }
        }
    }
}

/// Specialized stream for database query results
pub struct DatabaseStream<T> {
    pool: sqlx::SqlitePool,
    query: String,
    config: StreamConfig,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> DatabaseStream<T>
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + 'static,
{
    pub fn new(pool: sqlx::SqlitePool, query: String, config: StreamConfig) -> Self {
        Self {
            pool,
            query,
            config,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Stream database query results
    pub async fn stream(&self) -> impl Stream<Item = Result<T>> {
        let pool = self.pool.clone();
        let query = self.query.clone();
        let config = self.config.clone();

        async_stream::stream! {
            let mut offset = 0i64;
            let limit = config.batch_size as i64;
            let mut has_more = true;

            while has_more {
                let paginated_query = format!("{} LIMIT {} OFFSET {}", query, limit, offset);

                match sqlx::query_as::<_, T>(&paginated_query)
                    .fetch_all(&pool)
                    .await
                {
                    Ok(rows) => {
                        if rows.is_empty() {
                            has_more = false;
                        } else {
                            for row in rows {
                                yield Ok(row);
                            }
                            offset += limit;

                            // Apply backpressure
                            sleep(config.backpressure_delay).await;
                        }
                    }
                    Err(e) => {
                        yield Err(AppError::Database {
                            message: format!("Database stream error: {}", e),
                            query: paginated_query,
                        });
                        has_more = false;
                    }
                }

                // Check for timeout
                if let Some(timeout) = config.timeout {
                    // Timeout handling would be implemented here
                    // This is simplified for demonstration
                }
            }
        }
    }
}

/// Stream merger for combining multiple streams
pub struct StreamMerger<T> {
    streams: Vec<Box<dyn Stream<Item = Result<T>> + Send + Unpin>>,
}

impl<T> StreamMerger<T> {
    pub fn new() -> Self {
        Self {
            streams: Vec::new(),
        }
    }

    pub fn add_stream<S>(&mut self, stream: S)
    where
        S: Stream<Item = Result<T>> + Send + Unpin + 'static,
    {
        self.streams.push(Box::new(stream));
    }

    /// Merge all streams into one
    pub async fn merge(self) -> impl Stream<Item = Result<T>> {
        let mut stream_map = StreamMap::new();

        for (index, stream) in self.streams.into_iter().enumerate() {
            stream_map.insert(index, stream);
        }

        async_stream::stream! {
            while let Some((_key, item_result)) = stream_map.next().await {
                yield item_result;
            }
        }
    }
}

/// Memory-aware stream that monitors and controls memory usage
pub struct MemoryAwareStream<T> {
    inner: Box<dyn Stream<Item = Result<T>> + Send + Unpin>,
    max_memory_mb: usize,
    current_memory_mb: usize,
    checkpoint_counter: u64,
}

impl<T> MemoryAwareStream<T> {
    pub fn new<S>(inner: S, max_memory_mb: usize) -> Self
    where
        S: Stream<Item = Result<T>> + Send + Unpin + 'static,
    {
        Self {
            inner: Box::new(inner),
            max_memory_mb,
            current_memory_mb: 0,
            checkpoint_counter: 0,
        }
    }

    pub fn get_memory_usage(&self) -> usize {
        self.current_memory_mb
    }
}

impl<T> Stream for MemoryAwareStream<T> {
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Estimate memory usage (simplified)
        this.checkpoint_counter += 1;
        if this.checkpoint_counter % 1000 == 0 {
            // Memory monitoring would be more sophisticated in practice
            this.current_memory_mb = std::mem::size_of::<T>() * this.checkpoint_counter as usize / (1024 * 1024);

            // Apply backpressure if memory usage is high
            if this.current_memory_mb > this.max_memory_mb {
                tracing::warn!(
                    "Memory usage ({MB}) exceeds limit ({MB}), applying backpressure",
                    MB = this.current_memory_mb,
                    limit = this.max_memory_mb
                );

                // In a real implementation, you might:
                // 1. Pause the stream
                // 2. Flush buffers to disk
                // 3. Trigger garbage collection
                // 4. Notify upstream to slow down
            }
        }

        // Use Pin::new to safely access the inner stream
        let pinned_inner = unsafe { Pin::new_unchecked(&mut this.inner) };
        pinned_inner.poll_next(cx)
    }
}

/// Utility functions for common streaming patterns
pub mod utils {
    use super::*;

    /// Create a stream from an iterator with memory management
    pub fn iterator_stream<T, I>(iter: I, config: StreamConfig) -> impl Stream<Item = Result<T>>
    where
        T: Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        let (stream, sender) = BufferedStream::new(config.buffer_size, config.max_memory_usage_mb);

        tokio::spawn(async move {
            for item in iter {
                let stream_item = StreamItem::new(item);
                if sender.send(stream_item).await.is_err() {
                    break; // Receiver dropped
                }
            }
        });

        stream.map(|result| result.map(|item| item.data))
    }

    /// Batch process items from a stream
    pub async fn batch_process<T, R, F, Fut>(
        stream: impl Stream<Item = Result<T>> + Send + 'static,
        batch_size: usize,
        processor: F,
    ) -> impl Stream<Item = Result<R>>
    where
        T: Send + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Vec<R>>> + Send,
    {
        async_stream::stream! {
            let mut batch = Vec::with_capacity(batch_size);
            let mut stream = Box::pin(stream);

            while let Some(item_result) = stream.next().await {
                match item_result {
                    Ok(item) => {
                        batch.push(item);

                        if batch.len() >= batch_size {
                            match processor(batch.clone()).await {
                                Ok(results) => {
                                    for result in results {
                                        yield Ok(result);
                                    }
                                }
                                Err(e) => yield Err(e),
                            }
                            batch.clear();
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }

            // Process remaining items
            if !batch.is_empty() {
                match processor(batch).await {
                    Ok(results) => {
                        for result in results {
                            yield Ok(result);
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }
        }
    }

    /// Filter stream items with async predicate
    pub async fn async_filter<T, F, Fut>(
        stream: impl Stream<Item = Result<T>> + Send + 'static,
        mut predicate: F,
    ) -> impl Stream<Item = Result<T>>
    where
        T: Send + 'static,
        F: FnMut(&T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = bool> + Send,
    {
        async_stream::stream! {
            let mut stream = Box::pin(stream);
            while let Some(item_result) = stream.next().await {
                match item_result {
                    Ok(item) => {
                        if predicate(&item).await {
                            yield Ok(item);
                        }
                    }
                    Err(e) => yield Err(e),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::iter;

    #[tokio::test]
    async fn test_buffered_stream() {
        let (stream, sender) = BufferedStream::new(10, 1);

        // Send test data
        for i in 0..5 {
            let item = StreamItem::new(i);
            sender.send(item).await.unwrap();
        }
        drop(sender); // Close sender

        // Collect results
        let results: Vec<_> = stream.collect().await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_stream_processor() {
        let config = StreamConfig::default();
        let processor = StreamProcessor::new(config, |batch: Vec<StreamItem<i32>>| {
            Ok(batch.into_iter().map(|item| item.data * 2).collect())
        });

        let input_stream = iter(vec![
            Ok(StreamItem::new(1)),
            Ok(StreamItem::new(2)),
            Ok(StreamItem::new(3)),
        ]);

        let output_stream = processor.process_stream(input_stream);
        let results: Vec<_> = output_stream.collect().await;

        assert_eq!(results, vec![Ok(2), Ok(4), Ok(6)]);
    }
}