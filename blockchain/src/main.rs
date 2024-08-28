pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use types::BlockHeightTrait;

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Height: BlockHeightTrait;
}

fn main() {}
