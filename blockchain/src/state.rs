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

#[cfg(test)]
mod tests {

    mod new_state {
        mod success {
            use crate::state::State;

            #[test]
            fn test_new_state() {
                let state = State::new();
                assert!(state.data.is_empty());
            }
        }

        mod failure {
            use crate::state::State;

            #[test]
            fn test_new_state() {
                let state = State::new();
                assert!(!state.data.contains_key(&vec![1, 2, 3]));
            }
        }
    }

    mod insert_get {
        mod success {
            use crate::state::State;

            #[test]
            fn test_insert_and_get() {
                let mut state = State::new();
                let key = vec![1, 2, 3];
                let value = vec![4, 5, 6];

                assert!(state.insert(key.clone(), value.clone()).is_ok());
                assert_eq!(state.get(key), Some(&value));
            }
        }

        mod failure {
            use crate::state::State;

            #[test]
            fn test_insert_and_get() {
                let mut state = State::new();
                let key = vec![1, 2, 3];
                let value = vec![4, 5, 6];

                assert!(state.insert(key.clone(), value.clone()).is_ok());
                assert_ne!(state.get(vec![1, 2, 4]), Some(&value));
            }
        }
    }

    mod insert_overwrite {
        mod success {
            use crate::state::State;

            #[test]
            fn test_insert_overwrite() {
                let mut state = State::new();
                let key = vec![1, 2, 3];
                let value1 = vec![4, 5, 6];
                let value2 = vec![7, 8, 9];

                assert!(state.insert(key.clone(), value1).is_ok());
                assert!(state.insert(key.clone(), value2.clone()).is_ok());
                assert_eq!(state.get(key), Some(&value2));
            }
        }

        mod failure {
            use crate::state::State;

            #[test]
            fn test_insert_overwrite() {
                let mut state = State::new();
                let key = vec![1, 2, 3];
                let value1 = vec![4, 5, 6];
                let value2 = vec![7, 8, 9];

                assert!(state.insert(key.clone(), value1.clone()).is_ok());
                assert!(state.insert(key.clone(), value2.clone()).is_ok());
                assert_ne!(state.get(key), Some(&value1));
            }
        }
    }

    mod get_nonexistent {
        mod success {
            use crate::state::State;

            #[test]
            fn test_get_nonexistent() {
                let state = State::new();
                let key = vec![1, 2, 3];

                assert_eq!(state.get(key), None);
            }
        }

        mod failure {
            use crate::state::State;

            #[test]
            fn test_get_nonexistent() {
                let mut state = State::new();
                let key = vec![1, 2, 3];
                let value = vec![4, 5, 6];

                assert!(state.insert(key.clone(), value.clone()).is_ok());
                assert_ne!(state.get(vec![1, 2, 4]), Some(&value));
            }
        }
    }

    mod multi_insert {
        mod success {
            use crate::state::State;

            #[test]
            fn test_multiple_inserts() {
                let mut state = State::new();
                let pairs = vec![
                    (vec![1], vec![10]),
                    (vec![2], vec![20]),
                    (vec![3], vec![30]),
                ];

                for (key, value) in pairs.iter() {
                    assert!(state.insert(key.clone(), value.clone()).is_ok());
                }

                for (key, value) in pairs.iter() {
                    assert_eq!(state.get(key.clone()), Some(value));
                }
            }
        }

        mod failure {
            use crate::state::State;

            #[test]
            fn test_multiple_inserts() {
                let mut state = State::new();
                let pairs = vec![
                    (vec![1], vec![10]),
                    (vec![2], vec![20]),
                    (vec![3], vec![30]),
                ];

                let bad_pairs = vec![
                    (vec![4], vec![10]),
                    (vec![5], vec![20]),
                    (vec![6], vec![30]),
                ];

                for (key, value) in pairs.iter() {
                    assert!(state.insert(key.clone(), value.clone()).is_ok());
                }

                for (key, value) in bad_pairs.iter() {
                    assert_ne!(state.get(key.clone()), Some(value));
                }
            }
        }
    }
}
