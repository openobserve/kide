//! RAII-based cleanup patterns for proper resource management
//!
//! This module provides standardized cleanup mechanisms using RAII (Resource Acquisition Is Initialization)
//! to ensure resources are properly cleaned up when they go out of scope or when explicit cleanup is needed.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use std::collections::HashMap;
use async_trait::async_trait;

use crate::errors::{AppResult, AppError, SystemError};

/// Trait for components that need cleanup
#[async_trait]
pub trait Cleanup: Send + Sync {
    /// Perform cleanup operations
    async fn cleanup(&self) -> AppResult<()>;
    
    /// Check if cleanup has been performed
    fn is_cleaned_up(&self) -> bool;
    
    /// Get a human-readable name for this component
    fn component_name(&self) -> &str;
}

/// RAII guard for automatic cleanup on drop
pub struct CleanupGuard<T: Cleanup + 'static> {
    resource: Option<T>,
    auto_cleanup: bool,
}

impl<T: Cleanup + 'static> CleanupGuard<T> {
    /// Create a new cleanup guard with automatic cleanup on drop
    pub fn new(resource: T) -> Self {
        Self {
            resource: Some(resource),
            auto_cleanup: true,
        }
    }
    
    /// Create a new cleanup guard without automatic cleanup
    pub fn new_manual(resource: T) -> Self {
        Self {
            resource: Some(resource),
            auto_cleanup: false,
        }
    }
    
    /// Disable automatic cleanup on drop
    pub fn disable_auto_cleanup(&mut self) {
        self.auto_cleanup = false;
    }
    
    /// Enable automatic cleanup on drop
    pub fn enable_auto_cleanup(&mut self) {
        self.auto_cleanup = true;
    }
    
    /// Get a reference to the wrapped resource
    pub fn get(&self) -> Option<&T> {
        self.resource.as_ref()
    }
    
    /// Get a mutable reference to the wrapped resource
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.resource.as_mut()
    }
    
    /// Take ownership of the resource, disabling cleanup
    pub fn take(mut self) -> Option<T> {
        self.resource.take()
    }
    
    /// Perform cleanup manually and consume the guard
    pub async fn cleanup(mut self) -> AppResult<()> {
        if let Some(resource) = self.resource.take() {
            resource.cleanup().await?;
        }
        Ok(())
    }
}

impl<T: Cleanup + 'static> Drop for CleanupGuard<T> {
    fn drop(&mut self) {
        if self.auto_cleanup && self.resource.is_some() {
            if let Some(resource) = self.resource.take() {
                // Use tokio's Handle to spawn cleanup in background since we can't await in drop
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let component_name = resource.component_name().to_string();
                    handle.spawn(async move {
                        if let Err(e) = resource.cleanup().await {
                            eprintln!("⚠️  Cleanup failed for {}: {}", component_name, e);
                        } else {
                            println!("🧹 Cleaned up component: {}", component_name);
                        }
                    });
                } else {
                    eprintln!("⚠️  Cannot perform async cleanup for {} - no runtime available", resource.component_name());
                }
            }
        }
    }
}

/// Managed task handle with automatic cleanup
pub struct ManagedTask {
    handle: Option<JoinHandle<()>>,
    name: String,
    cleaned_up: AtomicBool,
}

impl ManagedTask {
    /// Create a new managed task
    pub fn new(handle: JoinHandle<()>, name: impl Into<String>) -> Self {
        Self {
            handle: Some(handle),
            name: name.into(),
            cleaned_up: AtomicBool::new(false),
        }
    }
    
    /// Check if the task is finished
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map_or(true, |h| h.is_finished())
    }
    
    /// Abort the task
    pub fn abort(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.abort();
            self.cleaned_up.store(true, Ordering::SeqCst);
        }
    }
}

#[async_trait]
impl Cleanup for ManagedTask {
    async fn cleanup(&self) -> AppResult<()> {
        if let Some(handle) = &self.handle {
            if !handle.is_finished() {
                handle.abort();
            }
            self.cleaned_up.store(true, Ordering::SeqCst);
        }
        Ok(())
    }
    
    fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(Ordering::SeqCst)
    }
    
    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Collection of managed resources with bulk cleanup
pub struct ResourceCollection {
    resources: Arc<Mutex<HashMap<String, Box<dyn Cleanup>>>>,
    name: String,
    cleaned_up: AtomicBool,
}

impl ResourceCollection {
    /// Create a new resource collection
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            resources: Arc::new(Mutex::new(HashMap::new())),
            name: name.into(),
            cleaned_up: AtomicBool::new(false),
        }
    }
    
    /// Add a resource to the collection
    pub async fn add_resource(&self, id: impl Into<String>, resource: Box<dyn Cleanup>) {
        let mut resources = self.resources.lock().await;
        resources.insert(id.into(), resource);
    }
    
    /// Remove a resource from the collection
    pub async fn remove_resource(&self, id: &str) -> Option<Box<dyn Cleanup>> {
        let mut resources = self.resources.lock().await;
        resources.remove(id)
    }
    
    /// Get count of managed resources
    pub async fn resource_count(&self) -> usize {
        let resources = self.resources.lock().await;
        resources.len()
    }
    
    /// Get list of resource IDs
    pub async fn resource_ids(&self) -> Vec<String> {
        let resources = self.resources.lock().await;
        resources.keys().cloned().collect()
    }
}

#[async_trait]
impl Cleanup for ResourceCollection {
    async fn cleanup(&self) -> AppResult<()> {
        let mut resources = self.resources.lock().await;
        let mut errors = Vec::new();
        
        for (id, resource) in resources.drain() {
            if let Err(e) = resource.cleanup().await {
                errors.push(format!("Resource '{}': {}", id, e));
            }
        }
        
        self.cleaned_up.store(true, Ordering::SeqCst);
        
        if !errors.is_empty() {
            return Err(AppError::System(SystemError::ResourceUnavailable {
                resource: self.name.clone(),
                message: format!("Cleanup errors: {}", errors.join("; ")),
            }));
        }
        
        Ok(())
    }
    
    fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(Ordering::SeqCst)
    }
    
    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Task manager for handling multiple background tasks
pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, ManagedTask>>>,
    name: String,
    cleaned_up: AtomicBool,
}

impl TaskManager {
    /// Create a new task manager
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            name: name.into(),
            cleaned_up: AtomicBool::new(false),
        }
    }
    
    /// Spawn and register a new task
    pub async fn spawn_task<F>(&self, id: impl Into<String>, name: impl Into<String>, future: F) -> AppResult<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let id = id.into();
        let handle = tokio::spawn(future);
        let managed_task = ManagedTask::new(handle, name);
        
        let mut tasks = self.tasks.write().await;
        tasks.insert(id, managed_task);
        
        Ok(())
    }
    
    /// Stop a specific task
    pub async fn stop_task(&self, id: &str) -> AppResult<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(mut task) = tasks.remove(id) {
            task.abort();
        }
        Ok(())
    }
    
    /// Get count of active tasks
    pub async fn active_task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.values().filter(|task| !task.is_finished()).count()
    }
    
    /// Get list of active task IDs
    pub async fn active_task_ids(&self) -> Vec<String> {
        let tasks = self.tasks.read().await;
        tasks.iter()
            .filter(|(_, task)| !task.is_finished())
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Clean up finished tasks
    pub async fn clean_finished_tasks(&self) -> usize {
        let mut tasks = self.tasks.write().await;
        let initial_count = tasks.len();
        
        tasks.retain(|_id, task| !task.is_finished());
        
        initial_count - tasks.len()
    }
}

#[async_trait]
impl Cleanup for TaskManager {
    async fn cleanup(&self) -> AppResult<()> {
        let mut tasks = self.tasks.write().await;
        let mut errors = Vec::new();
        
        for (id, mut task) in tasks.drain() {
            task.abort();
            if let Err(e) = task.cleanup().await {
                errors.push(format!("Task '{}': {}", id, e));
            }
        }
        
        self.cleaned_up.store(true, Ordering::SeqCst);
        
        if !errors.is_empty() {
            return Err(AppError::System(SystemError::ResourceUnavailable {
                resource: self.name.clone(),
                message: format!("Task cleanup errors: {}", errors.join("; ")),
            }));
        }
        
        Ok(())
    }
    
    fn is_cleaned_up(&self) -> bool {
        self.cleaned_up.load(Ordering::SeqCst)
    }
    
    fn component_name(&self) -> &str {
        &self.name
    }
}

/// Utility functions for common cleanup patterns
pub mod utils {
    use super::*;
    
    /// Create a cleanup guard for a tokio task
    pub fn guard_task(handle: JoinHandle<()>, name: impl Into<String>) -> CleanupGuard<ManagedTask> {
        CleanupGuard::new(ManagedTask::new(handle, name))
    }
    
    /// Create a resource collection with cleanup guard
    pub fn guard_collection(name: impl Into<String>) -> CleanupGuard<ResourceCollection> {
        CleanupGuard::new(ResourceCollection::new(name))
    }
    
    /// Create a task manager with cleanup guard
    pub fn guard_task_manager(name: impl Into<String>) -> CleanupGuard<TaskManager> {
        CleanupGuard::new(TaskManager::new(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};
    
    #[derive(Debug)]
    struct TestResource {
        name: String,
        cleaned_up: AtomicBool,
    }
    
    impl TestResource {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                cleaned_up: AtomicBool::new(false),
            }
        }
    }
    
    #[async_trait]
    impl Cleanup for TestResource {
        async fn cleanup(&self) -> AppResult<()> {
            self.cleaned_up.store(true, Ordering::SeqCst);
            Ok(())
        }
        
        fn is_cleaned_up(&self) -> bool {
            self.cleaned_up.load(Ordering::SeqCst)
        }
        
        fn component_name(&self) -> &str {
            &self.name
        }
    }
    
    #[tokio::test]
    async fn test_cleanup_guard() {
        let resource = TestResource::new("test");
        let guard = CleanupGuard::new(resource);
        
        // Resource should not be cleaned up yet
        assert!(!guard.get().unwrap().is_cleaned_up());
        
        // Perform manual cleanup
        guard.cleanup().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_resource_collection() {
        let collection = ResourceCollection::new("test_collection");
        
        collection.add_resource("resource1", Box::new(TestResource::new("resource1"))).await;
        collection.add_resource("resource2", Box::new(TestResource::new("resource2"))).await;
        
        assert_eq!(collection.resource_count().await, 2);
        
        collection.cleanup().await.unwrap();
        assert!(collection.is_cleaned_up());
        assert_eq!(collection.resource_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_task_manager() {
        let manager = TaskManager::new("test_manager");
        
        // Spawn a short-lived task
        manager.spawn_task("task1", "test task 1", async {
            sleep(Duration::from_millis(10)).await;
        }).await.unwrap();
        
        // Spawn a long-lived task
        manager.spawn_task("task2", "test task 2", async {
            sleep(Duration::from_secs(10)).await;
        }).await.unwrap();
        
        assert_eq!(manager.active_task_count().await, 2);
        
        // Wait for first task to finish
        sleep(Duration::from_millis(20)).await;
        
        let cleaned = manager.clean_finished_tasks().await;
        assert_eq!(cleaned, 1);
        assert_eq!(manager.active_task_count().await, 1);
        
        // Cleanup should stop remaining tasks
        manager.cleanup().await.unwrap();
        assert!(manager.is_cleaned_up());
    }
    
    #[tokio::test]
    async fn test_managed_task() {
        let handle = tokio::spawn(async {
            sleep(Duration::from_secs(10)).await;
        });
        
        let mut task = ManagedTask::new(handle, "test_task");
        
        assert!(!task.is_finished());
        assert!(!task.is_cleaned_up());
        
        task.abort();
        
        assert!(task.is_cleaned_up());
    }
}