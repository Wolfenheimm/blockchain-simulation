pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod state;
pub mod stf;
pub mod types;

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Block;
    type Hash;
    type Number;
}

fn main() {}
