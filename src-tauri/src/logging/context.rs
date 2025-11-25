//! Log Context Management
//!
//! Implements context providers and scope management for structured logging.

use crate::logging::traits::*;
use crate::logging::error::LogError;
use crate::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::sync::Mutex as AsyncMutex;

/// Thread-local context provider
pub struct ThreadLocalContextProvider {
    context: std::cell::RefCell<LogContext>,
    metadata: ContextMetadata,
}

impl ThreadLocalContextProvider {
    pub fn new() -> Self {
        let metadata = ContextMetadata {
            name: "thread_local".to_string(),
            version: "1.0.0".to_string(),
            description: "Thread-local log context provider".to_string(),
            supports_nesting: true,
            max_depth: 100,
        };

        Self {
            context: std::cell::RefCell::new(LogContext::new()),
            metadata,
        }
    }
}

impl Default for ThreadLocalContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LogContextProvider for ThreadLocalContextProvider {
    fn get_context(&self) -> &LogContext {
        // Return a reference to the current context
        // Note: This is a simplified implementation
        unsafe {
            std::mem::transmute(self.context.borrow().deref())
        }
    }

    fn set_context(&self, context: LogContext) {
        *self.context.borrow_mut() = context;
    }

    fn update_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        updater(&mut self.context.borrow_mut());
    }

    fn push_scope(&self, scope: LogScope) {
        self.context.borrow_mut().push_scope(scope);
    }

    fn pop_scope(&self) -> Option<LogScope> {
        self.context.borrow_mut().pop_scope()
    }

    fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }
}

/// Async-aware context provider using tokio
pub struct AsyncContextProvider {
    context: Arc<AsyncMutex<LogContext>>,
    metadata: ContextMetadata,
}

impl AsyncContextProvider {
    pub fn new() -> Self {
        let metadata = ContextMetadata {
            name: "async".to_string(),
            version: "1.0.0".to_string(),
            description: "Async-aware log context provider".to_string(),
            supports_nesting: true,
            max_depth: 200,
        };

        Self {
            context: Arc::new(AsyncMutex::new(LogContext::new())),
            metadata,
        }
    }

    pub async fn get_context_async(&self) -> LogContext {
        self.context.lock().await.clone()
    }

    pub async fn update_context_async<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        let mut context = self.context.lock().await;
        updater(&mut context);
    }

    pub async fn push_scope_async(&self, scope: LogScope) {
        let mut context = self.context.lock().await;
        context.push_scope(scope);
    }

    pub async fn pop_scope_async(&self) -> Option<LogScope> {
        let mut context = self.context.lock().await;
        context.pop_scope()
    }
}

impl Default for AsyncContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl LogContextProvider for AsyncContextProvider {
    fn get_context(&self) -> &LogContext {
        // For sync operations, we need to get a snapshot
        // This is a limitation of the trait design for async contexts
        panic!("AsyncContextProvider requires async methods. Use get_context_async() instead.")
    }

    fn set_context(&self, _context: LogContext) {
        panic!("AsyncContextProvider requires async methods. Use update_context_async() instead.");
    }

    fn update_context<F>(&self, _updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        panic!("AsyncContextProvider requires async methods. Use update_context_async() instead.");
    }

    fn push_scope(&self, _scope: LogScope) {
        panic!("AsyncContextProvider requires async methods. Use push_scope_async() instead.");
    }

    fn pop_scope(&self) -> Option<LogScope> {
        panic!("AsyncContextProvider requires async methods. Use pop_scope_async() instead.");
    }

    fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }
}

/// Hierarchical context provider with nested scope support
pub struct HierarchicalContextProvider {
    contexts: Arc<RwLock<HashMap<String, LogContext>>>,
    current_context: Arc<RwLock<String>>,
    max_depth: usize,
    metadata: ContextMetadata,
}

impl HierarchicalContextProvider {
    pub fn new(max_depth: usize) -> Self {
        let metadata = ContextMetadata {
            name: "hierarchical".to_string(),
            version: "1.0.0".to_string(),
            description: "Hierarchical log context provider with nested scopes".to_string(),
            supports_nesting: true,
            max_depth,
        };

        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
            current_context: Arc::new(RwLock::new("default".to_string())),
            max_depth,
            metadata,
        }
    }

    pub fn create_context(&self, name: String) -> Result<()> {
        let mut contexts = self.contexts.write().unwrap();
        contexts.insert(name.clone(), LogContext::new());

        // Set as current if it's the first context
        if contexts.len() == 1 {
            *self.current_context.write().unwrap() = name;
        }

        Ok(())
    }

    pub fn set_current_context(&self, name: String) -> Result<()> {
        let contexts = self.contexts.read().unwrap();
        if contexts.contains_key(&name) {
            *self.current_context.write().unwrap() = name;
            Ok(())
        } else {
            Err(AppError::validation_error(
                format!("Context '{}' does not exist", name),
                None,
            ))
        }
    }

    fn get_current_context(&self) -> LogContext {
        let current_name = self.current_context.read().unwrap().clone();
        let contexts = self.contexts.read().unwrap();
        contexts.get(&current_name).cloned().unwrap_or_default()
    }

    fn update_current_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        let current_name = self.current_context.read().unwrap().clone();
        let mut contexts = self.contexts.write().unwrap();
        if let Some(context) = contexts.get_mut(&current_name) {
            updater(context);
        }
    }
}

impl Default for HierarchicalContextProvider {
    fn default() -> Self {
        Self::new(50)
    }
}

impl LogContextProvider for HierarchicalContextProvider {
    fn get_context(&self) -> &LogContext {
        // This is a limitation - we can't return a reference to something we own
        // In practice, this would need a different trait design
        panic!("HierarchicalContextProvider needs to return a cloned context. Use get_current_context() method instead.");
    }

    fn set_context(&self, context: LogContext) {
        let current_name = self.current_context.read().unwrap().clone();
        let mut contexts = self.contexts.write().unwrap();
        contexts.insert(current_name, context);
    }

    fn update_context<F>(&self, updater: F)
    where
        F: FnOnce(&mut LogContext),
    {
        self.update_current_context(updater);
    }

    fn push_scope(&self, scope: LogScope) {
        let current_name = self.current_context.read().unwrap().clone();
        let mut contexts = self.contexts.write().unwrap();
        if let Some(context) = contexts.get_mut(&current_name) {
            if context.scopes.len() < self.max_depth {
                context.push_scope(scope);
            } else {
                eprintln!("Warning: Maximum context depth ({}) exceeded", self.max_depth);
            }
        }
    }

    fn pop_scope(&self) -> Option<LogScope> {
        let current_name = self.current_context.read().unwrap().clone();
        let mut contexts = self.contexts.write().unwrap();
        if let Some(context) = contexts.get_mut(&current_name) {
            context.pop_scope()
        } else {
            None
        }
    }

    fn metadata(&self) -> &ContextMetadata {
        &self.metadata
    }
}

/// Context builder for convenient context construction
pub struct ContextBuilder {
    fields: HashMap<String, LogFieldValue>,
    scopes: Vec<LogScope>,
}

impl ContextBuilder {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
            scopes: Vec::new(),
        }
    }

    pub fn add_field<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<LogFieldValue>,
    {
        self.fields.insert(key.into(), value.into());
        self
    }

    pub fn add_fields<I, K, V>(mut self, fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<LogFieldValue>,
    {
        for (key, value) in fields {
            self.fields.insert(key.into(), value.into());
        }
        self
    }

    pub fn add_scope(mut self, scope: LogScope) -> Self {
        self.scopes.push(scope);
        self
    }

    pub fn add_scope_with_level(mut self, name: String, level: LogLevel) -> Self {
        self.scopes.push(LogScope::new(name, level));
        self
    }

    pub fn build(self) -> LogContext {
        let mut context = LogContext::new();
        context.add_fields(self.fields);
        for scope in self.scopes {
            context.push_scope(scope);
        }
        context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Context-aware scope guard for automatic scope management
pub struct ScopeGuard<'a> {
    provider: &'a dyn LogContextProvider,
    popped_scope: bool,
}

impl<'a> ScopeGuard<'a> {
    pub fn new(provider: &'a dyn LogContextProvider, scope: LogScope) -> Self {
        provider.push_scope(scope);
        Self {
            provider,
            popped_scope: false,
        }
    }

    pub fn manual_pop(&mut self) -> Option<LogScope> {
        if !self.popped_scope {
            self.popped_scope = true;
            self.provider.pop_scope()
        } else {
            None
        }
    }
}

impl<'a> Drop for ScopeGuard<'a> {
    fn drop(&mut self) {
        if !self.popped_scope {
            self.provider.pop_scope();
        }
    }
}

/// Macro for convenient scoped logging
#[macro_export]
macro_rules! log_scope {
    ($provider:expr, $name:expr, $level:expr) => {
        $crate::logging::context::ScopeGuard::new(
            $provider,
            $crate::logging::traits::LogScope::new($name.to_string(), $level),
        )
    };
    ($provider:expr, $name:expr, $level:expr, $($field_key:expr => $field_value:expr),*) => {
        {
            let mut scope = $crate::logging::traits::LogScope::new($name.to_string(), $level);
            $(
                scope.add_field($field_key.to_string(), $field_value.into());
            )*
            $crate::logging::context::ScopeGuard::new($provider, scope)
        }
    };
}

/// Context middleware for request processing
pub struct RequestContextMiddleware {
    provider: Arc<dyn LogContextProvider>,
}

impl RequestContextMiddleware {
    pub fn new(provider: Arc<dyn LogContextProvider>) -> Self {
        Self { provider }
    }

    pub fn process_request(&self, request_id: String, user_id: Option<String>) -> Result<ScopeGuard> {
        let mut context_fields = HashMap::new();
        context_fields.insert("request_id".to_string(), LogFieldValue::String(request_id));

        if let Some(uid) = user_id {
            context_fields.insert("user_id".to_string(), LogFieldValue::String(uid));
        }

        let context = LogContext::with_fields(context_fields);
        self.provider.set_context(context);

        let scope = LogScope::new("request".to_string(), LogLevel::Info);
        Ok(ScopeGuard::new(self.provider.as_ref(), scope))
    }

    pub fn add_request_field(&self, key: String, value: LogFieldValue) {
        self.provider.update_context(|ctx| {
            ctx.add_field(key, value);
        });
    }
}

/// Context extractor for parsing structured information from log records
pub struct ContextExtractor {
    field_patterns: HashMap<String, String>,
    scope_patterns: Vec<String>,
}

impl ContextExtractor {
    pub fn new() -> Self {
        Self {
            field_patterns: HashMap::new(),
            scope_patterns: Vec::new(),
        }
    }

    pub fn add_field_pattern(mut self, field_name: String, pattern: String) -> Self {
        self.field_patterns.insert(field_name, pattern);
        self
    }

    pub fn add_scope_pattern(mut self, pattern: String) -> Self {
        self.scope_patterns.push(pattern);
        self
    }

    pub fn extract_from_message(&self, message: &str) -> LogContext {
        let mut context = LogContext::new();

        // Extract fields using patterns
        for (field_name, pattern) in &self.field_patterns {
            if let Some(captures) = self.extract_with_pattern(message, pattern) {
                for (i, capture) in captures.iter().enumerate() {
                    context.add_field(
                        format!("{}_{}", field_name, i),
                        LogFieldValue::String(capture.clone()),
                    );
                }
            }
        }

        // Extract scopes
        for pattern in &self.scope_patterns {
            if let Some(captures) = self.extract_with_pattern(message, pattern) {
                if let Some(scope_name) = captures.first() {
                    let scope = LogScope::new(scope_name.clone(), LogLevel::Info);
                    context.push_scope(scope);
                }
            }
        }

        context
    }

    fn extract_with_pattern(&self, message: &str, pattern: &str) -> Option<Vec<String>> {
        // Simple pattern matching - in a real implementation, this would use regex
        if message.contains(pattern) {
            let parts: Vec<&str> = message.split(pattern).collect();
            if parts.len() > 1 {
                Some(parts.iter().skip(1).map(|s| s.to_string()).collect())
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Default for ContextExtractor {
    fn default() -> Self {
        Self::new()
    }
}