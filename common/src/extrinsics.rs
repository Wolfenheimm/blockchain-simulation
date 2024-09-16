use crate::types::{Config, TransactionType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignedTransaction<T: Config> {
    pub transaction_type: TransactionType<T>,
}

impl<T: Config> Clone for SignedTransaction<T> {
    fn clone(&self) -> Self {
        Self {
            transaction_type: self.transaction_type.clone(),
        }
    }
}

impl<T: Config> SignedTransaction<T> {
    pub fn new(transaction_type: TransactionType<T>) -> Self {
        SignedTransaction { transaction_type }
    }

    pub fn weight(&self) -> T::WeightType {
        self.transaction_type.weight()
    }
}
