use serde::{Deserialize, Serialize};

use crate::types::Hash;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Account {
    pub account_id: Hash,
    pub balance: u128,
}
