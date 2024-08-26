use crate::block::{Block, BlockTrait};
use crate::extrinsics::ExtrinsicTrait;
use crate::Config;
use serde::{Deserialize, Serialize}; // Placeholder, perhaps bincode is better?
use std::error::Error;
use std::marker::PhantomData;

pub trait Stf<T: Config> {
    fn validate_block(&self, block: Block) -> Result<(), Box<dyn Error>>;
    fn execute_block(&self, block: Block);
}

pub struct SimpleStf<T: Config> {
    phanthom: PhantomData<T>,
}

impl<T: Config> SimpleStf<T> {
    pub fn new() -> Self {
        SimpleStf {
            phanthom: PhantomData::default(),
        }
    }
}

impl<T: Config> Stf<T> for SimpleStf<T> {
    fn validate_block(&self, block: Block) -> Result<(), Box<dyn Error>> {
        let parent_hash = block.parent_hash; // Use the config to call parent_hash

        // TODO: Check if the parent block exists

        // Ensure the block does not exceed its maximum weight
        let block_weight = calculate_weight(block);

        if block_weight > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&self, block: Block) {
        // TODO: Apply all extrinsic transactions to the state
        // Think about perhaps applying this to a layer of the state before pushing it.
    }
}

fn calculate_weight(block: impl BlockTrait) -> u64 {
    block.extrinsics().iter().map(|e| e.weight()).sum()
}
