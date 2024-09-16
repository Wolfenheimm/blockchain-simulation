use crate::State;
use common::types::StorageError;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;

pub trait StoragePlugin<P, K, V> {
    fn set(&mut self, prefix: P, key: K, value: &V) -> Result<(), StorageError>;
    fn get(&self, prefix: P, key: K) -> Result<V, StorageError>;
    fn create_full_key(prefix: P, key: K) -> Result<Vec<u8>, StorageError>;
}

#[derive(Serialize)]
pub struct Plugin {
    state: State,
}

impl Plugin {
    pub fn new() -> Self {
        Plugin {
            state: State::new(),
        }
    }

    // DEBUGGING
    pub fn get_state(&self) -> &State {
        &self.state
    }
}

impl<P, K, V> StoragePlugin<P, K, V> for Plugin
where
    P: Serialize + Debug,
    K: Serialize + Debug,
    V: Serialize + DeserializeOwned + Debug,
{
    fn set(&mut self, prefix: P, key: K, value: &V) -> Result<(), StorageError> {
        let full_key = <Self as StoragePlugin<P, K, V>>::create_full_key(prefix, key).unwrap();

        let encoded_value = bincode::serialize(value).unwrap();
        self.state
            .insert(full_key, encoded_value)
            .map_err(|e| StorageError::DataInsertionError(format!("{:?}", e)))?;

        Ok(())
    }

    fn get(&self, prefix: P, key: K) -> Result<V, StorageError> {
        let full_key = <Self as StoragePlugin<P, K, V>>::create_full_key(prefix, key)
            .map_err(|e| StorageError::KeyCreationError(format!("{:?}", e)))?;

        let encoded_data = self
            .state
            .get(full_key.clone())
            .ok_or_else(|| StorageError::KeyNotFound(format!("{:?}", full_key)))?;

        bincode::deserialize(encoded_data).map_err(|e| {
            eprintln!("Failed to deserialize data: {}", e);
            StorageError::DeserializationError(e.to_string())
        })
    }

    fn create_full_key(prefix: P, key: K) -> Result<Vec<u8>, StorageError> {
        let encoded_prefix = bincode::serialize(&prefix).map_err(|e| {
            eprintln!("Failed to serialize prefix: {}", e);
            StorageError::SerializationError("Failed to serialize prefix".to_string())
        })?;

        let encoded_key = bincode::serialize(&key).map_err(|e| {
            eprintln!("Failed to serialize key: {}", e);
            StorageError::SerializationError("Failed to serialize key".to_string())
        })?;

        let full_key = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();

        Ok(full_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod plugin_creation {
        mod success {
            use super::super::*;

            #[test]
            fn test_plugin_creation() {
                let plugin = Plugin::new();
                let input: Vec<u8> = vec![];
                assert!(plugin.get_state().get(input).is_none());
            }
        }
    }

    mod set_and_get {
        mod success {
            use super::super::*;

            #[test]
            fn test_set_and_get() {
                let mut plugin = Plugin::new();
                let prefix = "prefix";
                let key = "key";
                let value = 42u32;

                // Set the value
                let set_result = plugin.set(prefix, key, &value);
                assert!(set_result.is_ok());

                // Get the value
                let get_result: Result<u32, StorageError> = plugin.get(prefix, key);
                assert!(get_result.is_ok());
                assert_eq!(get_result.unwrap(), value);
            }
        }

        mod failure {
            use super::super::*;

            #[test]
            fn test_get_nonexistent_key() {
                let plugin = Plugin::new();
                let prefix = "prefix";
                let key = "dne_key";

                let get_result: Result<u32, StorageError> = plugin.get(prefix, key);
                assert!(matches!(get_result, Err(StorageError::KeyNotFound(_))));
            }
        }
    }

    mod create_full_key {
        mod success {
            use super::super::*;

            #[test]
            fn test_create_full_key() {
                let prefix = "test_prefix";
                let key = "test_key";

                let full_key_result =
                    <Plugin as StoragePlugin<_, _, u32>>::create_full_key(prefix, key);
                assert!(full_key_result.is_ok());

                let full_key = full_key_result.unwrap();
                assert!(!full_key.is_empty());
            }
        }
    }

    mod multiple_operations {
        mod success {
            use super::super::*;

            #[test]
            fn test_set_and_get_multiple_values() {
                let mut plugin = Plugin::new();
                let prefix = "test_prefix";
                let keys = vec!["key1", "key2", "key3"];
                let values = vec![1u32, 2u32, 3u32];

                // Set multiple values
                for (key, value) in keys.iter().zip(values.iter()) {
                    let set_result = plugin.set(prefix, *key, value);
                    assert!(set_result.is_ok());
                }

                // Get and verify multiple values
                for (key, expected_value) in keys.iter().zip(values.iter()) {
                    let get_result: Result<u32, StorageError> = plugin.get(prefix, *key);
                    assert!(get_result.is_ok());
                    assert_eq!(get_result.unwrap(), *expected_value);
                }
            }

            #[test]
            fn test_overwrite_value() {
                let mut plugin = Plugin::new();
                let prefix = "test_prefix";
                let key = "test_key";
                let initial_value = 42u32;
                let new_value = 84u32;

                // Set initial value
                let set_result = plugin.set(prefix, key, &initial_value);
                assert!(set_result.is_ok());

                // Overwrite with new value
                let set_result = plugin.set(prefix, key, &new_value);
                assert!(set_result.is_ok());

                // Get and verify the new value
                let get_result: Result<u32, StorageError> = plugin.get(prefix, key);
                assert!(get_result.is_ok());
                assert_eq!(get_result.unwrap(), new_value);
            }

            #[test]
            fn test_different_prefixes() {
                let mut plugin = Plugin::new();
                let prefix1 = "prefix1";
                let prefix2 = "prefix2";
                let key = "same_key";
                let value1 = 42u32;
                let value2 = 84u32;

                // Set values with different prefixes
                let set_result1 = plugin.set(prefix1, key, &value1);
                assert!(set_result1.is_ok());
                let set_result2 = plugin.set(prefix2, key, &value2);
                assert!(set_result2.is_ok());

                // Get and verify values
                let get_result1: Result<u32, StorageError> = plugin.get(prefix1, key);
                assert!(get_result1.is_ok());
                assert_eq!(get_result1.unwrap(), value1);

                let get_result2: Result<u32, StorageError> = plugin.get(prefix2, key);
                assert!(get_result2.is_ok());
                assert_eq!(get_result2.unwrap(), value2);
            }
        }
    }
}
