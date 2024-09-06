use crate::types::TransactionType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SignedTransaction {
    pub transaction_type: TransactionType,
}

impl SignedTransaction {
    pub fn new(transaction_type: TransactionType) -> Self {
        SignedTransaction { transaction_type }
    }

    pub fn weight(&self) -> u64 {
        self.transaction_type.weight()
    }
}
