use crate::{
    extrinsics::ExtrinsicTrait,
    types::{BlockHeight, Hash},
};

#[derive(Debug, Clone)]
pub struct Block<T> {
    pub block_height: BlockHeight,
    pub parent_hash: Hash,
    pub state_root: Hash,
    pub extrinsics: Vec<T>, // Generic E allows for different transaction types...
}

impl<E: ExtrinsicTrait> Block<E> {
    pub fn extrinsics(&self) -> &Vec<E> {
        &self.extrinsics
    }
}

pub trait BlockTrait {
    type Extrinsic: ExtrinsicTrait; // Associated type for extrinsics

    fn block_height(&self) -> BlockHeight;
    fn parent_hash(&self) -> Hash;
    fn state_root(&self) -> Hash;
    fn extrinsics(&self) -> &Vec<Self::Extrinsic>;
}

// Implement the BlockTrait for the Block struct
impl<T: ExtrinsicTrait> BlockTrait for Block<T> {
    type Extrinsic = T;

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
