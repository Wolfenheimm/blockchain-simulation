use crate::block::{Block, BlockTrait};
use crate::extrinsics::ExtrinsicTrait;
use crate::plugin::{Plugin, StoragePlugin};
use crate::state::State;
use crate::types::BlockHeight;
use crate::Config;
use serde::Serialize;
use std::error::Error;
use std::marker::PhantomData;

#[derive(Serialize)]
enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
}

pub trait Stf<T: Config> {
    fn validate_block(&self, block: Block, state: State) -> Result<(), Box<dyn Error>>;
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
    fn validate_block(&self, block: Block, state: State) -> Result<(), Box<dyn Error>> {
        // Ensure the block is not already in the state
        let block_key = Plugin::encode_key(StoragePrefix::Block, block.block_height);

        // TODO: Check if the parent block exists from State
        let parent_block_key =
            StoragePlugin::encode_key(StoragePrefix::Block, block.block_height - 1);

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

fn get_block(block_height: BlockHeight) -> Block {
    // Placeholder for getting the block from the state
    unimplemented!()
}
