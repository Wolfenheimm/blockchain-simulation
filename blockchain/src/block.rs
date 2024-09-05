use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};

use crate::{
    extrinsics::SignedTransaction,
    types::{BlockHeight, BlockWeight, Hash},
};

const MAX_BLOCK_WEIGHT: BlockWeight = 1000;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: Header,
    pub extrinsics: Vec<SignedTransaction>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Header {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics_root: Hash,
    pub block_weight: BlockWeight,
}

pub trait BlockTrait {
    fn extrinsics(&self) -> &Vec<SignedTransaction>;
    fn hash(&self) -> [u8; 32];
    fn add_extrinsic(
        &mut self,
        extrinsic: SignedTransaction,
        weight: BlockWeight,
    ) -> Result<(), String>;
    fn can_add_extrinsic(&self, weight: BlockWeight) -> bool;
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

    fn can_add_extrinsic(&self, weight: BlockWeight) -> bool {
        self.header.block_weight + weight <= MAX_BLOCK_WEIGHT
    }

    // Method to add an extrinsic if it does not exceed the maximum block weight
    fn add_extrinsic(
        &mut self,
        extrinsic: SignedTransaction,
        weight: BlockWeight,
    ) -> Result<(), String> {
        if self.can_add_extrinsic(weight) {
            self.extrinsics.push(extrinsic);
            self.header.block_weight += weight;
            Ok(())
        } else {
            Err(format!(
                "Block weight exceeded. Max allowed: {}, Current: {}, New Extrinsic: {}",
                MAX_BLOCK_WEIGHT, self.header.block_weight, weight
            ))
        }
    }
}
