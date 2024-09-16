use crate::types::Config;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Account<T: Config> {
    pub account_id: T::Hash,
    pub balance: T::Funds,
}

impl<T: Config> Clone for Account<T> {
    fn clone(&self) -> Self {
        Self {
            account_id: self.account_id.clone(),
            balance: self.balance.clone(),
        }
    }
}
