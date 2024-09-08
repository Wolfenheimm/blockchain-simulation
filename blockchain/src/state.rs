use serde::Serialize;
use std::collections::HashMap;

use crate::types::StorageError;

#[derive(Serialize)]
pub struct State {
    data: HashMap<Vec<u8>, Vec<u8>>,
}

impl State {
    pub fn get(&self, key: Vec<u8>) -> Option<&Vec<u8>> {
        self.data.get(&key)
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Result<(), StorageError> {
        self.data.insert(key, value);
        Ok(())
    }

    pub fn new() -> Self {
        State {
            data: HashMap::new(),
        }
    }

    // DEBUGGING
    pub fn print_state(&self) {
        println!("--- State Dump ---");
        for (key, value) in &self.data {
            let key_str = hex::encode(key);
            let value_str = hex::encode(value);

            println!("Key:");
            println!("  {}", key_str);
            println!("Value:");
            println!("  {}", value_str);
            println!("----------------------------------------------------");
        }
        println!("--- End of State Dump ---");
    }
}
