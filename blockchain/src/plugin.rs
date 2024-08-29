use std::marker::PhantomData;

use crate::{state::State, Config};
use serde::{de::DeserializeOwned, Serialize};

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
}

// TODO: Think about adding an overlay to state

impl<P, K, V> StoragePlugin<P, K, V> for Plugin
where
    P: Serialize,
    K: Serialize,
    V: Serialize + DeserializeOwned,
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
            let decoded: V = bincode::deserialize(&encoded_data[..]).unwrap();
            Some(decoded)
        } else {
            None
        }
    }
}
