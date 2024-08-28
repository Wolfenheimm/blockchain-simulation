use crate::block::{Block, BlockTrait};
use crate::plugin::{Plugin, StoragePlugin};
use crate::types::TransactionType;
use crate::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize)]
enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
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
        // ^^^ This is a potential fork scenario
        // TODO: This could potentially be a trigger event which would check consensus and fetch the accepted chain

        // Ensure the block does not exceed its maximum weight
        if calculate_weight(block) > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&self, block: Block, mut plugin: Plugin) {
        // Apply the block to the state
        plugin.encode(StoragePrefix::Block, &block.block_height, &block);

        // TODO: Apply all extrinsic transactions to the state
        // Think about perhaps applying this to a layer (or a new instance) of the state before squashing it?
        for transaction in block.extrinsics() {
            // Apply the transaction to the state
            plugin.encode(StoragePrefix::Extrinsic, &block.block_height, transaction);

            // Apply the transaction to the Account, then update state
        }
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
