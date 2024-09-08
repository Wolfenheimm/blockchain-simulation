use crate::account::Account;
use crate::block::{Block, BlockTrait};
use crate::plugin::{Plugin, StoragePlugin};
use crate::types::{StfError, StorageError, TransactionType};
use crate::Config;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;

// TODO: Add this to types, rename to StorageType
#[derive(Serialize, Deserialize, Debug)]
pub enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
}

pub trait Stf<T: Config>
where
    T: Serialize + DeserializeOwned + Debug,
{
    fn validate_block(&mut self, block: Block<T>) -> Result<(), Box<dyn Error>>;
    fn execute_block(&mut self, block: Block<T>) -> Result<(), StfError>;
    fn validate_account(&mut self, account: Account<T>) -> Result<(), Box<dyn Error>>;
    fn get_block_hash(&self, block_height: T::HeightType) -> Result<T::Hash, StorageError>;
    fn get_account(&self, account_id: T::Hash) -> Result<Account<T>, StorageError>;
}

pub struct SimpleStf<T: Config>
where
    T: Serialize + DeserializeOwned + Debug,
{
    plugin: Plugin,
    phantom: PhantomData<T>,
}

impl<T: Config> SimpleStf<T>
where
    T: Serialize + DeserializeOwned + Debug,
{
    pub fn new(plugin: Plugin) -> Self {
        SimpleStf {
            plugin,
            phantom: PhantomData,
        }
    }
}

impl<T: Config> Stf<T> for SimpleStf<T>
where
    T: Serialize + Debug + DeserializeOwned,
{
    fn validate_block(&mut self, block: Block<T>) -> Result<(), Box<dyn Error>> {
        // Ensure the block is not already in the state
        let block_exists: Result<(), StorageError> = self
            .plugin
            .get(StoragePrefix::Block, block.header.block_height.clone());
        // If exists... big no-no
        if block_exists.is_ok() {
            return Err("Block already exists in the state.".into());
        }

        // Check if the parent block exists from State
        let parent_block_key: Result<T::Hash, StorageError> = self.plugin.get(
            StoragePrefix::Block,
            block.header.block_height - T::HeightType::from(1),
        );

        // If parent block does not exist... big no-no
        if parent_block_key.is_err() {
            return Err("Parent block does not exist.".into());
        }

        // Check if the parent hash matches the parent block hash
        if block.header.parent_hash != parent_block_key.unwrap() {
            return Err("Parent hash is invalid for this block.".into());
        }
        // ^^^ This is a potential fork scenario
        // TODO: This could potentially be a trigger event which would check consensus and fetch the accepted chain

        Ok(())
    }

    fn execute_block(&mut self, block: Block<T>) -> Result<(), StfError> {
        // Add the block to the state. B# -> BH & BH -> B
        println!("BLOCK HEIGHT: {}", &block.header.block_height);
        let block_hash = block.hash();
        self.plugin
            .set(
                StoragePrefix::Block,
                &block.header.block_height,
                &block_hash,
            )
            .map_err(StfError::Storage)?;
        self.plugin.set(StoragePrefix::Block, &block_hash, &block)?;

        for transaction in block.extrinsics() {
            // Apply the transaction, then update state
            match transaction.transaction_type {
                TransactionType::Transfer {
                    amount, from, to, ..
                } => {
                    // Get the sender and receiver accounts
                    let from_account: Result<Account<T>, StorageError> =
                        self.plugin.get(StoragePrefix::Account, from);
                    let to_account: Result<Account<T>, StorageError> =
                        self.plugin.get(StoragePrefix::Account, to);

                    // Check if the accounts exist, if they don't, skip the transaction
                    if from_account.is_err() {
                        eprintln!("Sender account does not exist.");
                        continue;
                    }

                    if to_account.is_err() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, skip the transaction
                    if from_account.clone().unwrap().balance < amount {
                        eprintln!("Sender does not have enough balance.");
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account: Account<T> = Account {
                        account_id: from_account.clone().unwrap().account_id,
                        balance: from_account.clone().unwrap().balance - amount,
                    };
                    // Push
                    self.plugin
                        .set(
                            StoragePrefix::Account,
                            &from_account.unwrap().account_id,
                            &updated_from_account,
                        )
                        .map_err(StfError::Storage)?;

                    // Update the receiver's account
                    let updated_to_account: Account<T> = Account {
                        account_id: to_account.clone().unwrap().account_id,
                        balance: to_account.clone().unwrap().balance + amount,
                    };
                    // Push
                    self.plugin
                        .set(
                            StoragePrefix::Account,
                            &to_account.unwrap().account_id,
                            &updated_to_account,
                        )
                        .map_err(StfError::Storage)?;
                }
                TransactionType::Mint { amount, to, .. } => {
                    // Get the receiver's account
                    let to_account: Result<Account<T>, StorageError> =
                        self.plugin.get(StoragePrefix::Account, to);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if to_account.is_err() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Update the receiver's account
                    let updated_to_account: Account<T> = Account {
                        account_id: to_account.clone().unwrap().account_id,
                        balance: to_account.clone().unwrap().balance + amount,
                    };
                    // Push
                    self.plugin
                        .set(
                            StoragePrefix::Account,
                            to_account.unwrap().account_id,
                            &updated_to_account,
                        )
                        .map_err(StfError::Storage)?;
                }
                TransactionType::Burn { amount, from, .. } => {
                    // Get the sender's account
                    let from_account: Result<Account<T>, StorageError> =
                        self.plugin.get(StoragePrefix::Account, from);

                    // Check if the account exists, if it doesn't, skip the transaction
                    if from_account.is_err() {
                        eprintln!("Receiver account does not exist.");
                        continue;
                    }

                    // Check if the sender has enough balance, if they don't, set them to zero???
                    // TODO: Ask about this
                    if from_account.clone().unwrap().balance < amount {
                        eprintln!("Receiver does not have enough balance to burn that amount...");
                        continue;
                    }

                    // Update the sender's account
                    let updated_from_account: Account<T> = Account {
                        account_id: from_account.clone().unwrap().account_id,
                        balance: from_account.clone().unwrap().balance - amount,
                    };
                    // Push
                    self.plugin
                        .set(
                            StoragePrefix::Account,
                            from_account.unwrap().account_id,
                            &updated_from_account,
                        )
                        .map_err(StfError::Storage)?;
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
                    match self.validate_account(account.clone()) {
                        Ok(_) => {
                            // Add the account to the state
                            self.plugin
                                .set(StoragePrefix::Account, account.clone().account_id, &account)
                                .map_err(StfError::Storage)?;
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
            )?;
        }
        Ok(())
    }

    fn validate_account(&mut self, account: Account<T>) -> Result<(), Box<dyn Error>> {
        // Check if the account is not already in the state
        let account_exists: Result<Account<T>, StorageError> =
            self.plugin.get(StoragePrefix::Account, account.account_id);
        // If the account exists... big no-no
        if account_exists.is_ok() {
            return Err("Account already exists in the state.".into());
        }

        Ok(())
    }

    fn get_block_hash(&self, block_height: T::HeightType) -> Result<T::Hash, StorageError> {
        self.plugin.get(StoragePrefix::Block, block_height)
    }

    fn get_account(&self, account_id: T::Hash) -> Result<Account<T>, StorageError> {
        self.plugin.get(StoragePrefix::Account, account_id)
    }
}
