use crate::types::Hash;

#[derive(Debug, Clone, Copy)]
struct Account {
    hash: Hash,
    balance: u128,
}

pub type AccountId = Hash;
