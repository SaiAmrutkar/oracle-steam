// crates/oracle-callbacks/src/call_results.rs
// Call results are for async operations (like API calls that return SteamAPICall_t)

use crate::types::CallbackData;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Manages async call results
pub struct CallResultManager {
    results: Arc<RwLock<HashMap<u64, CallResultData>>>,
    next_handle: Arc<RwLock<u64>>,
}

struct CallResultData {
    data: Box<dyn std::any::Any + Send + Sync>,
    completed: bool,
    failed: bool,
}

impl CallResultManager {
    pub fn new() -> Self {
        Self {
            results: Arc::new(RwLock::new(HashMap::new())),
            next_handle: Arc::new(RwLock::new(1)),
        }
    }

    /// Create a new call result and return its handle
    pub fn create<T: CallbackData + 'static>(&self, data: T) -> u64 {
        let mut next = self.next_handle.write();
        let handle = *next;
        *next += 1;
        drop(next);

        let result_data = CallResultData {
            data: Box::new(data),
            completed: true,
            failed: false,
        };

        self.results.write().insert(handle, result_data);
        handle
    }

    /// Create a pending call result (not yet completed)
    pub fn create_pending(&self) -> u64 {
        let mut next = self.next_handle.write();
        let handle = *next;
        *next += 1;
        drop(next);

        // Create empty placeholder
        let result_data = CallResultData {
            data: Box::new(()),
            completed: false,
            failed: false,
        };

        self.results.write().insert(handle, result_data);
        handle
    }

    /// Complete a pending call result with data
    pub fn complete<T: CallbackData + 'static>(&self, handle: u64, data: T) {
        if let Some(result) = self.results.write().get_mut(&handle) {
            result.data = Box::new(data);
            result.completed = true;
            result.failed = false;
        }
    }

    /// Fail a call result
    pub fn fail(&self, handle: u64) {
        if let Some(result) = self.results.write().get_mut(&handle) {
            result.completed = true;
            result.failed = true;
        }
    }

    /// Check if a call is completed
    pub fn is_completed(&self, handle: u64) -> bool {
        self.results
            .read()
            .get(&handle)
            .map(|r| r.completed)
            .unwrap_or(false)
    }

    /// Check if a call failed
    pub fn is_failed(&self, handle: u64) -> bool {
        self.results
            .read()
            .get(&handle)
            .map(|r| r.failed)
            .unwrap_or(false)
    }

    /// Get the result data (consumes the result)
    pub fn get<T: CallbackData + Clone + 'static>(&self, handle: u64) -> Option<T> {
        let mut results = self.results.write();
        results.remove(&handle).and_then(|result| {
            if result.failed {
                return None;
            }
            result.data.downcast::<T>().ok().map(|b| (*b).clone())
        })
    }

    /// Peek at result data without consuming
    pub fn peek<T: CallbackData + Clone + 'static>(&self, handle: u64) -> Option<T> {
        let results = self.results.read();
        results.get(&handle).and_then(|result| {
            if result.failed {
                return None;
            }
            result.data.downcast_ref::<T>().map(|r| r.clone())
        })
    }

    /// Remove a call result
    pub fn remove(&self, handle: u64) -> bool {
        self.results.write().remove(&handle).is_some()
    }

    /// Clear all results
    pub fn clear(&self) {
        self.results.write().clear();
    }

    /// Get count of pending results
    pub fn count(&self) -> usize {
        self.results.read().len()
    }
}

impl Default for CallResultManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper trait for call result types
pub trait CallResult: CallbackData + Clone + 'static {
    /// The error type for this call result
    type Error;

    /// Check if this result represents success
    fn is_success(&self) -> bool;

    /// Convert to a Result type
    fn to_result(&self) -> Result<Self, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LobbyCreated_t;

    #[test]
    fn test_create_and_get() {
        let manager = CallResultManager::new();

        let data = LobbyCreated_t {
            result: 1,
            lobby_id: 123456,
        };

        let handle = manager.create(data.clone());
        assert!(manager.is_completed(handle));
        assert!(!manager.is_failed(handle));

        let retrieved = manager.get::<LobbyCreated_t>(handle);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().lobby_id, 123456);

        // Should be consumed
        assert!(!manager.is_completed(handle));
    }

    #[test]
    fn test_pending_completion() {
        let manager = CallResultManager::new();

        let handle = manager.create_pending();
        assert!(!manager.is_completed(handle));

        manager.complete(
            handle,
            LobbyCreated_t {
                result: 1,
                lobby_id: 999,
            },
        );

        assert!(manager.is_completed(handle));

        let data = manager.get::<LobbyCreated_t>(handle);
        assert!(data.is_some());
        assert_eq!(data.unwrap().lobby_id, 999);
    }

    #[test]
    fn test_failure() {
        let manager = CallResultManager::new();

        let handle = manager.create_pending();
        manager.fail(handle);

        assert!(manager.is_completed(handle));
        assert!(manager.is_failed(handle));

        let data = manager.get::<LobbyCreated_t>(handle);
        assert!(data.is_none());
    }

    #[test]
    fn test_peek() {
        let manager = CallResultManager::new();

        let data = LobbyCreated_t {
            result: 1,
            lobby_id: 777,
        };

        let handle = manager.create(data);

        // Peek doesn't consume
        let peeked = manager.peek::<LobbyCreated_t>(handle);
        assert!(peeked.is_some());
        assert_eq!(peeked.unwrap().lobby_id, 777);

        // Should still be there
        assert!(manager.is_completed(handle));

        // Get consumes
        let retrieved = manager.get::<LobbyCreated_t>(handle);
        assert!(retrieved.is_some());
        assert!(!manager.is_completed(handle));
    }

    #[test]
    fn test_clear() {
        let manager = CallResultManager::new();

        for i in 0..10 {
            manager.create(LobbyCreated_t {
                result: 1,
                lobby_id: i,
            });
        }

        assert_eq!(manager.count(), 10);
        manager.clear();
        assert_eq!(manager.count(), 0);
    }
}
