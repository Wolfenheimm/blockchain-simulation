use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];
pub type BlockHeight = u64;
pub type BlockWeight = u64;
pub struct MaxBlockWeightImpl;
pub struct MaxBlockHeightImpl;

pub trait HashTrait {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Self;
}

// Implement HashTrait for Hash type
impl HashTrait for Hash {
    fn as_bytes(&self) -> &[u8] {
        self
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes[..32]);
        hash
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer {
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
    },
    Mint {
        to: [u8; 32],
        amount: u128,
    },
    Burn {
        from: [u8; 32],
        amount: u128,
    },
    AccountCreation {
        account_id: [u8; 32],
        balance: u128,
    },
}

impl TransactionType {
    pub fn weight(&self) -> u64 {
        match self {
            Self::Transfer { .. } => 1,
            Self::Mint { .. } => 1,
            Self::Burn { .. } => 1,
            Self::AccountCreation { .. } => 1,
        }
    }
}

// TODO: Implement it, may have issues with current Option usage
#[derive(Debug)]
pub enum TransactionError {
    AccountNotFound(Hash), // To specify which account was not found
    InsufficientBalance {
        account_id: Hash,
        balance: u128,
        amount: u128,
    },
}
