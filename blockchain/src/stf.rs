use crate::blockchain::Blockchain;
use serde::{Deserialize, Serialize}; // Placeholder, perhaps bincode is better?
use std::error::Error;

pub trait Config {
    type Block;
    const MAX_BLOCK_WEIGHT: u64;
}

pub trait Stf<E, B: Blockchain<E>> {
    fn validate_block(&self, block: &B::Block) -> Result<(), Box<dyn Error>>;

    fn execute_block(&self, block: &B::Block);
}

pub struct SimpleStf;

impl<E, B> Stf<E, B> for SimpleStf
where
    E: serde::Serialize + serde::de::DeserializeOwned,
    B: Blockchain<E>,
{
    fn validate_block(&self, block: &B::Block) -> Result<(), Box<dyn Error>> {
        // TODO: Ensure parent block exists
        Ok(())
    }

    fn execute_block(&self, block: &B::Block) {
        // TODO: Apply all extrinsic transactions into the state
        // Think about perhaps applying this to a layer of the state before pushing it.
    }
}
