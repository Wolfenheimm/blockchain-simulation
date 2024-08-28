use serde::{de::DeserializeOwned, Serialize};
use std::collections::HashMap;

pub trait StoragePlugin<P, K, V> {
    fn encode(prefix: P, key: K, value: V) -> (Vec<u8>, Vec<u8>);
    fn decode(prefix: P, key: K, data: HashMap<Vec<u8>, Vec<u8>>) -> V;
}

pub trait KeyEncoder<P, K> {
    fn encode_key(prefix: P, key: K) -> Vec<u8>;
}

#[derive(Serialize)]
pub struct Plugin;

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

    fn decode(prefix: P, key: K, data: HashMap<Vec<u8>, Vec<u8>>) -> V {
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

impl<P, K> KeyEncoder<P, K> for Plugin
where
    P: Serialize,
    K: Serialize,
{
    fn encode_key(prefix: P, key: K) -> Vec<u8> {
        let encoded_prefix = bincode::serialize(&prefix).unwrap();
        let encoded_key = bincode::serialize(&key).unwrap();
        let full_key: Vec<_> = encoded_prefix
            .into_iter()
            .chain(encoded_key.into_iter())
            .collect();

        full_key
    }
}
