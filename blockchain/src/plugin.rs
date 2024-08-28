use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

use crate::state::State;

pub trait StoragePlugin<P, K, V> {
    fn encode(prefix: P, key: K, value: V) -> (Vec<u8>, Vec<u8>);
    fn get(&self, prefix: P, key: K) -> V;
}

#[derive(Serialize)]
pub struct Plugin {
    state: State,
}

impl<P, K, V> StoragePlugin<P, K, V> for Plugin
where
    P: Serialize,
    K: Serialize,
    V: Serialize + DeserializeOwned,
{
    fn encode(prefix: P, key: K, value: V) -> (Vec<u8>, Vec<u8>) {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let new_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        let encoded_value = bincode::serialize(&value).unwrap();

        (new_key, encoded_value)
    }

    fn get(&self, prefix: P, key: K) -> V {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let full_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        let encoded_data: Vec<u8> = self.state.get(full_key).clone().unwrap().to_vec();
        let decoded: V = bincode::deserialize(&encoded_data[..]).unwrap();

        decoded
    }
}
