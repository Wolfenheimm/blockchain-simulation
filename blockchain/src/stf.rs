use crate::account::Account;
use crate::block::{Block, BlockTrait};
use crate::plugin::{Plugin, StoragePlugin};
use crate::types::TransactionType;
use crate::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;

// TODO: Add this to types, rename to StorageType
#[derive(Serialize, Deserialize)]
pub enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
}

pub trait Stf<T: Config> {
    fn validate_block(&self, block: Block, plugin: &mut Plugin) -> Result<(), Box<dyn Error>>;
    fn execute_block(&self, block: Block, plugin: &mut Plugin);
    fn validate_account(&self, account: Account, plugin: &mut Plugin)
        -> Result<(), Box<dyn Error>>;
    fn add_account(&self, account: Account, plugin: &mut Plugin);
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
    fn validate_block(&self, block: Block, plugin: &mut Plugin) -> Result<(), Box<dyn Error>> {
        // Ensure the block is not already in the state
        let block_exists: Option<()> = plugin.get(StoragePrefix::Block, block.header.block_height);
        // If exists... big no-no
        if block_exists.is_some() {
            return Err("Block already exists in the state.".into());
        }
        // TODO: perhaps extra validation on block hash found in header

        // Check if the parent block exists from State
        let parent_block_key: Option<()> =
            plugin.get(StoragePrefix::Block, block.header.block_height - 1);
        // If parent block does not exist... big no-no
        if parent_block_key.is_none() {
            return Err("Parent block is invalid for this block.".into());
        }
        // TODO: Add a check for the parent hash

        // ^^^ This is a potential fork scenario
        // TODO: This could potentially be a trigger event which would check consensus and fetch the accepted chain

        // Ensure the block does not exceed its maximum weight
        if calculate_weight(block) > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&self, block: Block, plugin: &mut Plugin) {
        // Add the block to the state. B# -> BH & BH -> B
        let block_hash = block.hash();
        plugin.set(
            StoragePrefix::Block,
            &block.header.block_height,
            &block_hash,
        );
        plugin.set(StoragePrefix::Block, &block_hash, &block);

        for transaction in block.extrinsics() {
            // Apply the transaction, then update state
            match transaction.transaction_type {
                TransactionType::Transfer {
                    amount, from, to, ..
                } => {
                    // Get the sender's account
                    let from_account: Option<Account> = plugin.get(StoragePrefix::Account, from);
                    // Get the receiver's account
                    let to_account: Option<Account> = plugin.get(StoragePrefix::Account, to);

                    // TODO: explore the use of ? for these Options

                    // Check if the accounts exist, if they don't, skip the transaction
                    if from_account.is_none() || to_account.is_none() {
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, skip the transaction
                    if from_account.unwrap().balance < amount {
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account = Account {
                        account_id: from_account.unwrap().account_id,
                        balance: from_account.unwrap().balance - amount,
                    };
                    // Push
                    plugin.set(
                        StoragePrefix::Account,
                        &from_account.unwrap().account_id,
                        &updated_from_account,
                    );

                    // Update the receiver's account
                    let updated_to_account = Account {
                        account_id: to_account.unwrap().account_id,
                        balance: to_account.unwrap().balance + amount,
                    };
                    // Push
                    plugin.set(
                        StoragePrefix::Account,
                        &to_account.unwrap().account_id,
                        &updated_to_account,
                    );
                }
                TransactionType::Mint { amount, to, .. } => {
                    // Get the receiver's account
                    let to_account: Option<Account> = plugin.get(StoragePrefix::Account, to);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if to_account.is_none() {
                        continue;
                    }

                    // Update the receiver's account
                    let updated_to_account = Account {
                        account_id: to_account.unwrap().account_id,
                        balance: to_account.unwrap().balance + amount,
                    };
                    // Push
                    plugin.set(
                        StoragePrefix::Account,
                        &to_account.unwrap().account_id,
                        &updated_to_account,
                    );
                }
                TransactionType::Burn { amount, from, .. } => {
                    // Get the sender's account
                    let from_account: Option<Account> = plugin.get(StoragePrefix::Account, from);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if from_account.is_none() {
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, set them to zero???
                    // TODO: Ask about this
                    if from_account.unwrap().balance < amount {
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account = Account {
                        account_id: from_account.unwrap().account_id,
                        balance: from_account.unwrap().balance - amount,
                    };
                    // Push
                    plugin.set(
                        StoragePrefix::Account,
                        &from_account.unwrap().account_id,
                        &updated_from_account,
                    );
                }
            }

            // TODO: If something happened, think about a rollback...
            // Add the transaction to the state
            plugin.set(
                StoragePrefix::Extrinsic,
                &block.header.block_height,
                transaction,
            );
        }
    }

    fn validate_account(
        &self,
        account: Account,
        plugin: &mut Plugin,
    ) -> Result<(), Box<dyn Error>> {
        // Check if the account is not already in the state
        let account_exists: Option<Account> =
            plugin.get(StoragePrefix::Account, account.account_id);
        // If the account exists... big no-no
        if account_exists.is_some() {
            return Err("Account already exists in the state.".into());
        }

        Ok(())
    }

    fn add_account(&self, account: Account, plugin: &mut Plugin) {
        plugin.set(
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
            TransactionType::Mint { weight, .. } => *weight,
            TransactionType::Burn { weight, .. } => *weight,
        })
        .sum()
}
