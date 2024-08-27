use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct State {
    data: HashMap<Vec<u8>, Vec<u8>>,
}
