pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod state;
pub mod stf;
pub mod types;
use block::BlockTrait;
use extrinsics::ExtrinsicTrait;
use types::{BlockHeightTrait, HashTrait};

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Hash: HashTrait;
    type Height: BlockHeightTrait;
    type Extrinsic: ExtrinsicTrait;

    // fn fetch_block_by_hash(hash: &Self::Hash) -> Option<Self::Block>;
    // fn extrinsics_from_block(block: &Self::Block) -> &Vec<Self::Extrinsic>;
}

fn main() {}
