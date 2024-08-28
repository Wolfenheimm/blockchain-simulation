use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct State {
    data: HashMap<Vec<u8>, Vec<u8>>, // Encoded key-value pairs
}

impl State {
    pub fn get(&self, key: Vec<u8>) -> Option<&Vec<u8>> {
        self.data.get(&key)
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.data.insert(key, value);
    }
}
