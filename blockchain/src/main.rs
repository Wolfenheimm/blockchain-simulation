mod consensus;
mod stf;

pub trait Config {
    type Block;
    type Hash;
}

fn main() {
    println!("Hello, world!");
}
