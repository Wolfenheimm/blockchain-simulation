use crate::Config;
use serde::{Deserialize, Serialize}; // Placeholder, perhaps bincode is better?
use std::error::Error;

pub trait Stf<T: Config> {
    fn validate_block(&self, block: &T::Block) -> Result<(), Box<dyn Error>>;
    fn execute_block(&self, block: &T::Block);
}

pub struct SimpleStf;

impl<T: Config> Stf<T> for SimpleStf {
    fn validate_block(&self, block: &T::Block) -> Result<(), Box<dyn Error>> {
        // TODO: Ensure parent block exists. Look at this from the state -> stored as a hash of block_height, block_hash
        //       deserialize the parent block and check if the parent hash matches the parent block hash
        // TODO: Check if the block has reached its maximum weight of extrinsics, if it has, it's full. Execute it.
        Ok(())
    }

    fn execute_block(&self, block: &T::Block) {
        // TODO: Apply all extrinsic transactions to the state
        // Think about perhaps applying this to a layer of the state before pushing it.
    }
}
