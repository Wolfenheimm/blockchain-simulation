use crate::account::AccountId;
use std::collections::HashMap;

struct State {
    balances: HashMap<AccountId, u128>,
    total_issuance: u128,
}

impl State {
    // TODO: Encodes the state into a byte array.

    // TODO: Decodes the state from a byte array.
}
