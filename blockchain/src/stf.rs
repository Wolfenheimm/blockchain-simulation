use crate::account::Account;
use crate::block::{Block, BlockTrait};
use crate::plugin::{Plugin, StoragePlugin};
use crate::types::TransactionType;
use crate::Config;
use serde::{Deserialize, Serialize};
use std::error::Error;

// TODO: Add this to types, rename to StorageType
#[derive(Serialize, Deserialize, Debug)]
pub enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
}

pub trait Stf<T: Config> {
    fn validate_block(&mut self, block: Block) -> Result<(), Box<dyn Error>>;
    fn execute_block(&mut self, block: Block);
    fn validate_account(&mut self, account: Account) -> Result<(), Box<dyn Error>>;
    fn get_block_hash(&self, block_height: u64) -> Option<[u8; 32]>;
    fn get_account(&self, account_id: [u8; 32]) -> Option<Account>;
}

pub struct SimpleStf<T: Config> {
    config: T,
    plugin: Plugin,
}

impl<T: Config> SimpleStf<T> {
    pub fn new(config: T, plugin: Plugin) -> Self {
        SimpleStf { config, plugin }
    }
}

impl<T: Config> Stf<T> for SimpleStf<T> {
    fn validate_block(&mut self, block: Block) -> Result<(), Box<dyn Error>> {
        // Ensure the block is not already in the state
        let block_exists: Option<()> = self
            .plugin
            .get(StoragePrefix::Block, block.header.block_height);
        // If exists... big no-no
        if block_exists.is_some() {
            return Err("Block already exists in the state.".into());
        }

        // Check if the parent block exists from State
        let parent_block_key: Option<[u8; 32]> = self
            .plugin
            .get(StoragePrefix::Block, block.header.block_height - 1);

        // If parent block does not exist... big no-no
        if parent_block_key.is_none() {
            return Err("Parent block does not exist.".into());
        }

        // Check if the parent hash matches the parent block hash
        if block.header.parent_hash != parent_block_key.unwrap() {
            return Err("Parent hash is invalid for this block.".into());
        }
        // ^^^ This is a potential fork scenario
        // TODO: This could potentially be a trigger event which would check consensus and fetch the accepted chain

        // Ensure the block does not exceed its maximum weight
        if calculate_weight(block) > T::MAX_BLOCK_WEIGHT {
            return Err("Block exceeds maximum weight.".into());
        }

        Ok(())
    }

    fn execute_block(&mut self, block: Block) {
        // Add the block to the state. B# -> BH & BH -> B
        let block_hash = block.hash();
        self.plugin.set(
            StoragePrefix::Block,
            &block.header.block_height,
            &block_hash,
        );
        self.plugin.set(StoragePrefix::Block, &block_hash, &block);

        for transaction in block.extrinsics() {
            // Apply the transaction, then update state
            match transaction.transaction_type {
                TransactionType::Transfer {
                    amount, from, to, ..
                } => {
                    // Get the sender and receiver accounts
                    let from_account: Option<Account> =
                        self.plugin.get(StoragePrefix::Account, from);
                    let to_account: Option<Account> = self.plugin.get(StoragePrefix::Account, to);

                    // TODO: explore the use of ? for these Options -> TransactionError enum made for this

                    // Check if the accounts exist, if they don't, skip the transaction
                    if from_account.is_none() {
                        eprintln!("Sender account does not exist.");
                        continue;
                    }

                    if to_account.is_none() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, skip the transaction
                    if from_account.unwrap().balance < amount {
                        eprintln!("Sender does not have enough balance.");
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account = Account {
                        account_id: from_account.unwrap().account_id,
                        balance: from_account.unwrap().balance - amount,
                    };
                    // Push
                    self.plugin.set(
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
                    self.plugin.set(
                        StoragePrefix::Account,
                        &to_account.unwrap().account_id,
                        &updated_to_account,
                    );
                }
                TransactionType::Mint { amount, to, .. } => {
                    // Get the receiver's account
                    let to_account: Option<Account> = self.plugin.get(StoragePrefix::Account, to);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if to_account.is_none() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Update the receiver's account
                    let updated_to_account = Account {
                        account_id: to_account.unwrap().account_id,
                        balance: to_account.unwrap().balance + amount,
                    };
                    // Push
                    self.plugin.set(
                        StoragePrefix::Account,
                        &to_account.unwrap().account_id,
                        &updated_to_account,
                    );
                }
                TransactionType::Burn { amount, from, .. } => {
                    // Get the sender's account
                    let from_account: Option<Account> =
                        self.plugin.get(StoragePrefix::Account, from);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if from_account.is_none() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, set them to zero???
                    // TODO: Ask about this
                    if from_account.unwrap().balance < amount {
                        eprintln!("Receiver does not have enough balance to burn that amount...");
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account = Account {
                        account_id: from_account.unwrap().account_id,
                        balance: from_account.unwrap().balance - amount,
                    };
                    // Push
                    self.plugin.set(
                        StoragePrefix::Account,
                        &from_account.unwrap().account_id,
                        &updated_from_account,
                    );
                }
                TransactionType::AccountCreation {
                    account_id,
                    balance,
                    ..
                } => {
                    // Create the account
                    let account = Account {
                        account_id,
                        balance,
                    };

                    // Validate the account
                    match self.validate_account(account) {
                        Ok(_) => {
                            // Add the account to the state
                            self.plugin
                                .set(StoragePrefix::Account, &account.account_id, &account);
                        }
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                    }
                }
            }

            // TODO: If something happened, think about a rollback...
            // Add the transaction to the state
            self.plugin.set(
                StoragePrefix::Extrinsic,
                &block.header.block_height,
                transaction,
            );
        }
    }

    fn validate_account(&mut self, account: Account) -> Result<(), Box<dyn Error>> {
        // Check if the account is not already in the state
        let account_exists: Option<Account> =
            self.plugin.get(StoragePrefix::Account, account.account_id);
        // If the account exists... big no-no
        if account_exists.is_some() {
            return Err("Account already exists in the state.".into());
        }

        Ok(())
    }

    fn get_block_hash(&self, block_height: u64) -> Option<[u8; 32]> {
        self.plugin.get(StoragePrefix::Block, block_height)
    }

    fn get_account(&self, account_id: [u8; 32]) -> Option<Account> {
        self.plugin.get(StoragePrefix::Account, account_id)
    }
}

fn calculate_weight(block: impl BlockTrait) -> u64 {
    block.extrinsics().iter().map(|e| e.weight()).sum()
}
