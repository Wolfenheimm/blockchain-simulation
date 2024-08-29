use serde::{Deserialize, Serialize};

pub type Hash = [u8; 32];
pub type BlockHeight = u64;

pub trait HashTrait {
    fn as_bytes(&self) -> &[u8];
    fn from_bytes(bytes: &[u8]) -> Self;
}

// Define a trait for BlockHeight-related operations
pub trait BlockHeightTrait {
    fn value(&self) -> u64;
    fn from_value(value: u64) -> Self;
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

// Implement BlockHeightTrait for BlockHeight type
impl BlockHeightTrait for BlockHeight {
    fn value(&self) -> u64 {
        *self
    }

    fn from_value(value: u64) -> Self {
        value
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Transfer {
        weight: u64,
        from: [u8; 32],
        to: [u8; 32],
        amount: u128,
    },
    Mint {
        weight: u64,
        to: [u8; 32],
        amount: u128,
    },
    Burn {
        weight: u64,
        from: [u8; 32],
        amount: u128,
    },
}
