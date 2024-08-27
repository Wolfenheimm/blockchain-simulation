use crate::{
    extrinsics::{Extrinsic, ExtrinsicTrait},
    types::{BlockHeight, Hash},
};

#[derive(Debug, Clone)]
pub struct Block {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics: Vec<Extrinsic>, // Generic E allows for different transaction types...
}

pub trait BlockTrait {
    type Extrinsic: ExtrinsicTrait; // Associated type for extrinsics

    fn block_height(&self) -> BlockHeight;
    fn parent_hash(&self) -> Hash;
    fn state_root(&self) -> Hash;
    fn extrinsics(&self) -> &Vec<Self::Extrinsic>;
}

// Implement the BlockTrait for the Block struct
impl BlockTrait for Block {
    type Extrinsic = Extrinsic;

    fn block_height(&self) -> BlockHeight {
        self.block_height
    }

    fn parent_hash(&self) -> Hash {
        self.parent_hash
    }

    fn state_root(&self) -> Hash {
        self.state_root
    }

    fn extrinsics(&self) -> &Vec<Self::Extrinsic> {
        &self.extrinsics
    }
}
