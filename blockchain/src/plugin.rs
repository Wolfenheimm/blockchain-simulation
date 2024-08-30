use crate::state::State;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

pub trait StoragePlugin<P, K, V> {
    fn set(&mut self, prefix: P, key: K, value: &V);
    fn get(&self, prefix: P, key: K) -> Option<V>;
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
    P: Serialize,
    K: Serialize,
    V: Serialize + DeserializeOwned + Debug,
{
    fn set(&mut self, prefix: P, key: K, value: &V) {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let full_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        let encoded_value = bincode::serialize(value).unwrap();
        self.state.insert(full_key, encoded_value);
    }

    fn get(&self, prefix: P, key: K) -> Option<V> {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let full_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        if let Some(encoded_data) = self.state.get(full_key) {
            let decoded = bincode::deserialize(&encoded_data[..]).unwrap();
            println!("Decoded: {:?}", hex::encode(encoded_data));
            Some(decoded)
        } else {
            None
        }
    }
}
