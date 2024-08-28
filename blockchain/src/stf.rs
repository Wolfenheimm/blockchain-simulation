use crate::account::Account;
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
    fn validate_account(&self, account: Account, plugin: Plugin) -> Result<(), Box<dyn Error>>;
    fn add_account(&self, account: Account, plugin: Plugin);
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
            return Err("Parent block is invalid for this block.".into());
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
        // Add the block to the state
        plugin.encode(StoragePrefix::Block, &block.block_height, &block);

        // Add all extrinsics to the state
        // Apply all extrinsic transactions to the state
        for transaction in block.extrinsics() {
            // Apply the transaction to the state
            plugin.encode(StoragePrefix::Extrinsic, &block.block_height, transaction);

            // Apply the transaction, then update state
            match transaction.transaction_type {
                TransactionType::Transfer {
                    amount, from, to, ..
                } => {
                    // Get the sender's account
                    let from_account: Account = plugin.get(StoragePrefix::Account, from);
                    // Get the receiver's account
                    let to_account: Account = plugin.get(StoragePrefix::Account, to);

                    // TODO: Check if the accounts exist, if they don't, skip the transaction
                    // TODO: Check if the sender has enough balance, if they don't, skip the transaction

                    // Update the sender's account
                    let updated_from_account = Account {
                        account_id: from_account.account_id,
                        balance: from_account.balance - amount,
                    };
                    // Push
                    plugin.encode(
                        StoragePrefix::Account,
                        &from_account.account_id,
                        &updated_from_account,
                    );

                    // Update the receiver's account
                    let updated_to_account = Account {
                        account_id: to_account.account_id,
                        balance: to_account.balance + amount,
                    };
                    // Push
                    plugin.encode(
                        StoragePrefix::Account,
                        &to_account.account_id,
                        &updated_to_account,
                    );
                }
                TransactionType::None => {
                    // Bupkis
                }
            }
        }
    }

    fn validate_account(&self, account: Account, plugin: Plugin) -> Result<(), Box<dyn Error>> {
        // Check if the account is not already in the state
        let account_exists: Option<()> = plugin.get(StoragePrefix::Account, account.account_id);
        // If the account exists... big no-no
        if account_exists.is_some() {
            return Err("Account already exists in the state.".into());
        }

        Ok(())
    }

    fn add_account(&self, account: Account, mut plugin: Plugin) {
        plugin.encode(
            StoragePrefix::Account,
            &account.account_id,
            &account.balance,
        );
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
