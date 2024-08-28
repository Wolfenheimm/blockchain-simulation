use crate::block::{Block, BlockTrait};
use crate::extrinsics::Extrinsics;
use crate::plugin::{KeyEncoder, Plugin, StoragePlugin};
use crate::state::State;
use crate::types::{BlockHeight, TransactionType};
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
        state.get(block_key).ok_or("Block already exists.")?;

        // Check if the parent block exists from State
        let parent_block_key = Plugin::encode_key(StoragePrefix::Block, block.block_height - 1);
        state
            .get(parent_block_key)
            .ok_or("Parent block does not exist.")?;
        // This could potentially be a trigger event which would check consensus and fetch the accepted chain
        // Potential fork occurred...

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
    block
        .extrinsics()
        .iter()
        .map(|e| match &e.transaction_type {
            TransactionType::Transfer { weight, .. } => *weight,
            TransactionType::None => 0,
        })
        .sum()
}
