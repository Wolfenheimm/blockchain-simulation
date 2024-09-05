use crate::types::TransactionType;
use serde::{Deserialize, Serialize};

pub trait Extrinsics {
    type Extrinsic;

    fn add_extrinsic(&mut self, transaction_type: TransactionType);
}

impl Extrinsics for Vec<SignedTransaction> {
    type Extrinsic = SignedTransaction;

    fn add_extrinsic(&mut self, transaction_type: TransactionType) {
        let new_extrinsic = SignedTransaction::new(transaction_type);

        self.push(new_extrinsic)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct SignedTransaction {
    pub transaction_type: TransactionType,
}

impl SignedTransaction {
    pub fn new(transaction_type: TransactionType) -> Self {
        SignedTransaction { transaction_type }
    }

    pub fn weight(&self) -> u64 {
        match self.transaction_type {
            TransactionType::Transfer { weight, .. } => weight,
            TransactionType::Mint { weight, .. } => weight,
            TransactionType::Burn { weight, .. } => weight,
            TransactionType::AccountCreation { weight, .. } => weight,
        }
    }
}
