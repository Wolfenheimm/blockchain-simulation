use crate::state::State;
use crate::types::StorageError;
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

// TODO: Think about adding an overlay to state
// TODO: Blake2b hashing for keys

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
    mod create_full_key {
        mod success {
            use crate::{
                plugin::{Plugin, StoragePlugin},
                types::StorageError,
            };

            #[test]
            fn test_create_full_key() {
                let prefix = vec![1, 2, 3];
                let key = vec![1, 2, 3];

                let full_key = <Plugin as StoragePlugin<Vec<u8>, Vec<u8>, ()>>::create_full_key(
                    prefix.clone(),
                    key.clone(),
                )
                .map_err(|e| StorageError::KeyCreationError(format!("{:?}", e)))
                .unwrap();

                // Serialize prefix and key separately using bincode
                let encoded_prefix = bincode::serialize(&prefix).unwrap();
                let encoded_key = bincode::serialize(&key).unwrap();

                // Create the expected full key
                let expected_full_key: Vec<u8> = encoded_prefix
                    .clone()
                    .into_iter()
                    .chain(encoded_key.into_iter())
                    .collect();

                // Check if full_key matches the expected full key
                assert_eq!(full_key, expected_full_key);

                // Deserialize and check the prefix part
                let deserialized_prefix: Vec<u8> =
                    bincode::deserialize(&full_key[..encoded_prefix.len()]).unwrap();
                assert_eq!(deserialized_prefix, prefix);

                // Deserialize and check the key part
                let deserialized_key: Vec<u8> =
                    bincode::deserialize(&full_key[encoded_prefix.len()..]).unwrap();
                assert_eq!(deserialized_key, key);
            }
        }

        mod failure {}
    }

    mod get_data {
        mod success {}

        mod failure {}
    }

    mod set_data {
        mod success {}

        mod failure {}
    }
}
