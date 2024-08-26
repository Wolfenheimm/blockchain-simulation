use crate::extrinsics::ExtrinsicTrait;
use crate::Config;
use serde::{Deserialize, Serialize}; // Placeholder, perhaps bincode is better?
use std::error::Error;

pub trait Stf<T: Config> {
    fn validate_block(&self, block: &T::Block) -> Result<(), Box<dyn Error>>;
    fn execute_block(&self, block: &T::Block);
}

pub struct SimpleStf<T: Config> {
    config: T,
}

impl<T: Config> SimpleStf<T> {
    pub fn new(config: T) -> Self {
        SimpleStf { config }
    }
}

impl<T: Config> Stf<T> for SimpleStf<T> {
    fn validate_block(&self, block: &T::Block) -> Result<(), Box<dyn Error>> {
        // Get the parent hash using the config instance
        let parent_hash = T::parent_hash(block); // Use the config to call parent_hash
        let parent_block = T::fetch_block_by_hash(&parent_hash);

        // TODO: Check if the parent block exists

        // Ensure the block does not exceed its maximum weight
        let block_weight = calculate_weight::<T>(block);

        if block_weight > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&self, block: &T::Block) {
        // TODO: Apply all extrinsic transactions to the state
        // Think about perhaps applying this to a layer of the state before pushing it.
    }
}

fn calculate_weight<T>(block: &T::Block) -> u64
where
    T: Config,
    <T as Config>::Extrinsic: ExtrinsicTrait,
{
    T::extrinsics_from_block(block)
        .iter()
        .map(|e| e.weight())
        .sum()
}
