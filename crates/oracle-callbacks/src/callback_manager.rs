// crates/oracle-callbacks/src/callback_manager.rs
use crate::types::CallbackData;
use std::any::{Any, TypeId};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

type CallbackHandler = Box<dyn Fn(&dyn Any) + Send + Sync>;

pub struct CallbackManager {
    // Registered callback handlers by type
    handlers: HashMap<TypeId, Vec<CallbackHandler>>,
    // Pending callbacks to dispatch
    pending_callbacks: VecDeque<Box<dyn Any + Send + Sync>>,
    // Call results (async operations)
    call_results: HashMap<u64, Box<dyn Any + Send + Sync>>,
    next_call_handle: u64,
    initialized: bool,
}

impl CallbackManager {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            pending_callbacks: VecDeque::new(),
            call_results: HashMap::new(),
            next_call_handle: 1,
            initialized: false,
        }
    }

    pub fn initialize(&mut self) {
        if !self.initialized {
            println!("[Callbacks] System initialized");
            self.initialized = true;
        }
    }

    /// Register a callback handler for type T
    pub fn register_callback<T: CallbackData + 'static>(
        &mut self,
        handler: Box<dyn Fn(&T) + Send + Sync>,
    ) {
        let type_id = TypeId::of::<T>();

        // Wrap the typed handler in an untyped handler
        let wrapped_handler: CallbackHandler = Box::new(move |data: &dyn Any| {
            if let Some(typed_data) = data.downcast_ref::<T>() {
                handler(typed_data);
            }
        });

        self.handlers
            .entry(type_id)
            .or_insert_with(Vec::new)
            .push(wrapped_handler);
    }

    /// Queue a callback for dispatch
    pub fn queue_callback<T: CallbackData + 'static>(&mut self, data: T) {
        self.pending_callbacks.push_back(Box::new(data));
    }

    /// Dispatch all pending callbacks
    pub fn dispatch_callbacks(&mut self) {
        while let Some(callback_data) = self.pending_callbacks.pop_front() {
            let type_id = (*callback_data).type_id();

            if let Some(handlers) = self.handlers.get(&type_id) {
                for handler in handlers {
                    handler(callback_data.as_ref());
                }
            }
        }
    }

    /// Create an async call result
    pub fn create_call_result<T: CallbackData + 'static>(&mut self, data: T) -> u64 {
        let handle = self.next_call_handle;
        self.next_call_handle += 1;

        self.call_results.insert(handle, Box::new(data));

        handle
    }

    /// Check if call result is ready
    pub fn is_call_result_ready(&self, handle: u64) -> bool {
        self.call_results.contains_key(&handle)
    }

    /// Get call result data
    pub fn get_call_result<T: CallbackData + Clone + 'static>(&mut self, handle: u64) -> Option<T> {
        self.call_results
            .remove(&handle)
            .and_then(|boxed| boxed.downcast::<T>().ok().map(|b| *b))
    }

    /// Clear all callbacks and results
    pub fn clear(&mut self) {
        self.pending_callbacks.clear();
        self.call_results.clear();
    }

    /// Get pending callback count
    pub fn pending_count(&self) -> usize {
        self.pending_callbacks.len()
    }
}

impl Default for CallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::UserStatsReceived_t;
    use std::sync::RwLock;

    #[test]
    fn test_callback_dispatch() {
        let mut manager = CallbackManager::new();
        manager.initialize();

        let called = Arc::new(RwLock::new(false));
        let called_clone = called.clone();

        manager.register_callback::<UserStatsReceived_t>(Box::new(move |data| {
            assert_eq!(data.game_id, 480);
            *called_clone.write().unwrap() = true;
        }));

        manager.queue_callback(UserStatsReceived_t {
            game_id: 480,
            result: 1,
            steam_id: 12345,
        });

        assert_eq!(manager.pending_count(), 1);
        manager.dispatch_callbacks();
        assert_eq!(manager.pending_count(), 0);
        assert!(*called.read().unwrap());
    }

    #[test]
    fn test_multiple_handlers() {
        let mut manager = CallbackManager::new();

        let count = Arc::new(RwLock::new(0));
        let count1 = count.clone();
        let count2 = count.clone();

        manager.register_callback::<UserStatsReceived_t>(Box::new(move |_| {
            *count1.write().unwrap() += 1;
        }));

        manager.register_callback::<UserStatsReceived_t>(Box::new(move |_| {
            *count2.write().unwrap() += 10;
        }));

        manager.queue_callback(UserStatsReceived_t {
            game_id: 480,
            result: 1,
            steam_id: 12345,
        });

        manager.dispatch_callbacks();
        assert_eq!(*count.read().unwrap(), 11);
    }

    #[test]
    fn test_call_results() {
        let mut manager = CallbackManager::new();

        let data = UserStatsReceived_t {
            game_id: 730,
            result: 1,
            steam_id: 99999,
        };

        let handle = manager.create_call_result(data.clone());
        assert!(manager.is_call_result_ready(handle));

        let result = manager.get_call_result::<UserStatsReceived_t>(handle);
        assert!(result.is_some());
        assert_eq!(result.unwrap().game_id, 730);

        // Should be removed after retrieval
        assert!(!manager.is_call_result_ready(handle));
    }
}
