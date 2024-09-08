use crate::types::StorageError;
use crate::{state::State, types};
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
