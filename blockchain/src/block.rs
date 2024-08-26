use crate::{
    extrinsics::Extrinsic,
    types::{BlockHeight, Hash},
};

#[derive(Debug, Clone)]
pub struct Block<T> {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics: Vec<T>, // Generic E allows for different transaction types...
}

impl<E: Extrinsic> Block<E> {
    pub fn extrinsics(&self) -> &Vec<E> {
        &self.extrinsics
    }
}
