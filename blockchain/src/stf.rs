use crate::block::{Block, BlockTrait};
use crate::plugin::{Plugin, StoragePlugin};
use crate::types::TransactionType;
use crate::Config;
use serde::Serialize;
use std::error::Error;
use std::marker::PhantomData;

#[derive(Serialize)]
enum StoragePrefix {
    Account,
    Block,
    Extrinsics,
}

pub trait Stf<T: Config> {
    fn validate_block(&self, block: Block, plugin: Plugin) -> Result<(), Box<dyn Error>>;
    fn execute_block(&self, block: Block, plugin: Plugin);
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
    fn validate_block(&self, block: Block, plugin: Plugin) -> Result<(), Box<dyn Error>> {
        // Ensure the block is not already in the state
        let block_exists: Option<()> = plugin.get(StoragePrefix::Block, block.block_height);
        // If exists... big no-no
        if block_exists.is_some() {
            return Err("Block already exists in the state.".into());
        }

        // Check if the parent block exists from State
        let parent_block_key: Option<()> = plugin.get(StoragePrefix::Block, block.block_height - 1);
        // If parent block does not exist... big no-no
        if parent_block_key.is_none() {
            return Err("Parent block does not exist in the state.".into());
        }

        // TODO: This could potentially be a trigger event which would check consensus and fetch the accepted chain
        // Potential fork occurred...

        // Ensure the block does not exceed its maximum weight
        if calculate_weight(block) > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&self, block: Block, plugin: Plugin) {
        // TODO: Apply all extrinsic transactions to the state
        // Think about perhaps applying this to a layer of the state before pushing it.
        !unimplemented!()
    }
}

fn calculate_weight(block: impl BlockTrait) -> u64 {
    block
        .extrinsics()
        .iter()
        .map(|e| match &e.transaction_type {
            TransactionType::Transfer { weight, .. } => *weight,
            TransactionType::None => 0,
        })
        .sum()
}
