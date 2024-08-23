mod consensus;
mod stf;

pub trait Config {
    /// Type representing the block.
    type Block;
    /// Type representing the hash.
    type Hash;
    /// Type representing the block number.
    type Number;
}

fn main() {
    println!("Hello, world!");
}
