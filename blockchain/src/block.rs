use serde::{Deserialize, Serialize};

use crate::{
    extrinsics::SignedTransaction,
    types::{BlockHeight, Hash},
};

//TODO: Add a header to the block => add a block hash, timestamp, nonce, difficulty?
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics: Vec<SignedTransaction>,
}

pub trait BlockTrait {
    fn extrinsics(&self) -> &Vec<SignedTransaction>;
}

// Implement the BlockTrait for the Block struct
impl BlockTrait for Block {
    fn extrinsics(&self) -> &Vec<SignedTransaction> {
        &self.extrinsics
    }
}
