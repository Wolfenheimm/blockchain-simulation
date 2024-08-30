use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};

use crate::{
    extrinsics::SignedTransaction,
    types::{BlockHeight, Hash},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub extrinsics: Vec<SignedTransaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics_root: Hash,
}

pub trait BlockTrait {
    fn extrinsics(&self) -> &Vec<SignedTransaction>;
    fn hash(&self) -> [u8; 32];
}

// Implement the BlockTrait for the Block struct
impl BlockTrait for Block {
    fn extrinsics(&self) -> &Vec<SignedTransaction> {
        &self.extrinsics
    }

    fn hash(&self) -> [u8; 32] {
        let mut hasher = Blake2s256::new();
        hasher.update(self.header.block_height.to_le_bytes());
        hasher.update(self.header.parent_hash);
        hasher.update(self.header.state_root);
        hasher.update(self.header.extrinsics_root);
        hasher
            .finalize()
            .try_into()
            .expect("This hash has an expected size of 32 bytes")
    }
}
