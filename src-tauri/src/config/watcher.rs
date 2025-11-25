//! Configuration Watchers
//!
//! Implements file watching for configuration changes.

use crate::config::traits::*;
use crate::error::{AppError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// File configuration watcher
pub struct FileConfigWatcher {
    watchers: HashMap<WatcherId, Box<dyn ConfigChangeCallback>>,
    next_watcher_id: Arc<RwLock<WatcherId>>,
    status: Arc<RwLock<WatcherStatus>>,
    metadata: WatcherMetadata,
    file_path: String,
    last_modified: Arc<RwLock<Option<std::time::SystemTime>>>,
}

impl FileConfigWatcher {
    pub fn new(file_path: String) -> Self {
        let metadata = WatcherMetadata {
            name: "file".to_string(),
            version: "1.0.0".to_string(),
            description: format!("File watcher for {}", file_path),
            supported_sources: vec!["file".to_string()],
        };

        Self {
            watchers: HashMap::new(),
            next_watcher_id: Arc::new(RwLock::new(1)),
            status: Arc::new(RwLock::new(WatcherStatus::Stopped)),
            metadata,
            file_path,
            last_modified: Arc::new(RwLock::new(None)),
        }
    }

    async fn check_file_changes(&self) -> Result<Option<ConfigChangeEvent>> {
        let metadata = std::fs::metadata(&self.file_path)?;
        let modified = metadata.modified()?;

        let last_modified = *self.last_modified.read().await;

        if let Some(last) = last_modified {
            if modified > last {
                *self.last_modified.write().await = Some(modified);

                // Read the file content to determine what changed
                let content = tokio::fs::read_to_string(&self.file_path).await?;

                return Ok(Some(ConfigChangeEvent {
                    key: "file_content".to_string(),
                    old_value: None, // Would need to track previous content
                    new_value: Some(ConfigValue::String(content)),
                    change_type: ChangeType::Updated,
                    source: "file_system".to_string(),
                    timestamp: chrono::Utc::now(),
                    user_id: None,
                    session_id: None,
                    request_id: None,
                }));
            }
        } else {
            *self.last_modified.write().await = Some(modified);
        }

        Ok(None)
    }

    async fn notify_watchers(&mut self, event: ConfigChangeEvent) -> Result<()> {
        let mut failed_watchers = Vec::new();

        for (watcher_id, callback) in &mut self.watchers {
            match callback.on_change(event.clone()) {
                Ok(()) => {
                    tracing::debug!("Notified file watcher {} of configuration change", watcher_id);
                }
                Err(e) => {
                    tracing::error!("Failed to notify file watcher {}: {}", watcher_id, e);
                    failed_watchers.push(*watcher_id);
                }
            }
        }

        // Remove failed watchers
        for watcher_id in failed_watchers {
            self.watchers.remove(&watcher_id);
            tracing::warn!("Removed failed file watcher: {}", watcher_id);
        }

        Ok(())
    }
}

#[async_trait]
impl ConfigWatcher for FileConfigWatcher {
    async fn start(&mut self) -> Result<()> {
        *self.status.write().await = WatcherStatus::Starting;

        // Check if file exists
        if !std::path::Path::new(&self.file_path).exists() {
            return Err(AppError::Config {
                message: format!("Configuration file not found: {}", self.file_path),
                field: "file_path".to_string(),
            });
        }

        // Store initial modification time
        let metadata = std::fs::metadata(&self.file_path)?;
        let modified = metadata.modified()?;
        *self.last_modified.write().await = Some(modified);

        // Start background monitoring task
        let file_path = self.file_path.clone();
        let last_modified = self.last_modified.clone();
        let status = self.status.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));

            loop {
                interval.tick().await;

                let current_status = *status.read().await;
                if current_status != WatcherStatus::Running {
                    break;
                }

                if let Ok(metadata) = std::fs::metadata(&file_path) {
                    if let Ok(modified) = metadata.modified() {
                        let last = *last_modified.read().await;
                        if let Some(last_time) = last {
                            if modified > last_time {
                                *last_modified.write().await = Some(modified);

                                // File changed - in a real implementation, this would trigger callbacks
                                tracing::info!("Configuration file changed: {}", file_path);
                            }
                        }
                    }
                }
            }
        });

        *self.status.write().await = WatcherStatus::Running;
        tracing::info!("Started file watcher for: {}", self.file_path);

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        *self.status.write().await = WatcherStatus::Stopping;

        // The background task will stop when it checks the status
        tokio::time::sleep(Duration::from_millis(100)).await;

        *self.status.write().await = WatcherStatus::Stopped;
        tracing::info!("Stopped file watcher for: {}", self.file_path);

        Ok(())
    }

    fn subscribe(&mut self, callback: Box<dyn ConfigChangeCallback>) -> Result<WatcherId> {
        let mut next_id = self.next_watcher_id.write().unwrap();
        let watcher_id = *next_id;
        *next_id += 1;

        self.watchers.insert(watcher_id, callback);
        tracing::info!("Added file configuration watcher: {}", watcher_id);

        Ok(watcher_id)
    }

    fn unsubscribe(&mut self, watcher_id: WatcherId) -> Result<bool> {
        let removed = self.watchers.remove(&watcher_id).is_some();
        if removed {
            tracing::info!("Removed file configuration watcher: {}", watcher_id);
        }
        Ok(removed)
    }

    fn status(&self) -> WatcherStatus {
        // This would need to be async in real implementation
        // For now, return a placeholder
        WatcherStatus::Stopped
    }

    fn watcher_metadata(&self) -> &WatcherMetadata {
        &self.metadata
    }
}