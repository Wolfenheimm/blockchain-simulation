use crate::account::AccountId;
use crate::types::{BlockHeight, Hash};
use std::collections::HashMap;

struct State {
    blockchain: HashMap<BlockHeight, Hash>,
    balances: HashMap<AccountId, u128>,
    total_issuance: u128,
}

impl State {
    // TODO: Encodes the state into a byte array.

    // TODO: Decodes the state from a byte array.
}
