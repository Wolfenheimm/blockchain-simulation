mod account;
mod blockchain;
mod consensus;
mod extrinsics;
mod state;
mod stf;

pub trait Config {
    type Block;
    type Hash;
    type Number;
}

fn main() {}
