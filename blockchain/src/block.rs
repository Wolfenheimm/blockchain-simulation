use crate::types::{BlockHeight, Hash};

#[derive(Debug, Clone)]
pub struct Block<E> {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics: E, // Generic E allows for different transaction types...
}
