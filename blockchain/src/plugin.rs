use serde::de::DeserializeOwned;
use std::collections::HashMap;

pub trait StoragePlugin<P, V> {
    fn encode(&self, prefix: P, key: P, value: V) -> (Vec<u8>, Vec<u8>);
    fn decode(&self, prefix: P, key: P, data: HashMap<Vec<u8>, Vec<u8>>) -> V;
}

pub struct Plugin;

impl<P, V> StoragePlugin<P, V> for Plugin
where
    P: serde::Serialize,
    V: serde::Serialize + DeserializeOwned,
{
    fn encode(&self, prefix: P, key: P, value: V) -> (Vec<u8>, Vec<u8>) {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let new_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        let encoded_value = bincode::serialize(&value).unwrap();

        (new_key, encoded_value)
    }

    fn decode(&self, prefix: P, key: P, data: HashMap<Vec<u8>, Vec<u8>>) -> V {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let new_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();
        let encoded_data: Vec<u8> = data.get(&new_key).clone().unwrap().to_vec();
        let decoded: V = bincode::deserialize(&encoded_data[..]).unwrap();

        decoded
    }
}
