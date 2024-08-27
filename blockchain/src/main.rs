pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use extrinsics::ExtrinsicTrait;
use types::{BlockHeightTrait, HashTrait};

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Hash: HashTrait;
    type Height: BlockHeightTrait;
    type Extrinsic: ExtrinsicTrait;
}

fn main() {}
