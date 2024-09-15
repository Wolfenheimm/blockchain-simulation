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
                            &from_account.clone().unwrap().account_id,
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

    // Check if the account already exists, this validation is used for the account creation transaction
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::Account;
    use crate::block::Block;
    use crate::plugin::Plugin;
    use crate::types::{Height, MaxBlockHeight, MaxBlockWeight, StfError};

    // Mock implementation of Config trait for testing
    #[derive(Serialize, Deserialize, Debug)]
    struct MockConfig;
    impl Config for MockConfig {
        type MaxBlockWeight = MaxBlockWeight;
        type MaxBlockHeight = MaxBlockHeight;
        type WeightType = u64;
        type HeightType = Height;
        type Hash = [u8; 32];
        type Funds = u128;
    }

    mod validate_block {
        use super::*;

        mod success {
            use crate::block::Header;

            use super::*;

            #[test]
            fn test_validate_new_block() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                // Create a mock parent block and add it to the state
                let parent_block: Block<MockConfig> = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };
                assert!(stf.execute_block(parent_block.clone()).is_ok());

                // Create a new block
                let new_block = Block {
                    header: Header {
                        block_height: Height::from(2),
                        parent_hash: parent_block.hash(),
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                assert!(stf.validate_block(new_block).is_ok());
            }
        }

        mod failure {
            use crate::{block::Header, Zero};

            use super::*;

            #[test]
            fn test_validate_existing_block() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                let block = Block {
                    header: Header {
                        block_height: Height::zero(),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                // Place the block into the chain
                assert!(stf.execute_block(block.clone()).is_ok());

                // Validate it after it was placed, this should result in an error...
                // Validate is used before placing a block into the chain.
                assert!(stf.validate_block(block).is_err());
            }

            #[test]
            fn test_validate_block_with_missing_parent() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                // Create a block with a non-existent parent
                let block = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                assert!(stf.validate_block(block).is_err());
            }
        }
    }

    mod execute_block {
        use super::*;

        mod success {

            use crate::{
                block::Header,
                extrinsics::{self, SignedTransaction},
                types,
            };

            use super::*;

            #[test]
            fn test_execute_block_with_transfer() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                // Create a block with a transfer transaction
                let transaction =
                    extrinsics::SignedTransaction::new(types::TransactionType::Transfer {
                        from: [0; 32],
                        to: [1; 32],
                        amount: 30,
                    });

                let acc_alice: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [0; 32],
                        balance: 100,
                    });

                let acc_dave: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [1; 32],
                        balance: 50,
                    });

                let mut block = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                block.add_extrinsic(acc_alice).unwrap();
                block.add_extrinsic(acc_dave).unwrap();
                block.add_extrinsic(transaction).unwrap();

                assert!(stf.execute_block(block).is_ok());

                // Check updated balances
                let updated_from: Account<MockConfig> = stf
                    .get_account(<tests::MockConfig as Config>::Hash::from([0; 32]))
                    .unwrap();
                let updated_to: Account<MockConfig> = stf
                    .get_account(<tests::MockConfig as Config>::Hash::from([1; 32]))
                    .unwrap();

                assert_eq!(
                    updated_from.balance,
                    <tests::MockConfig as Config>::Funds::from(70u128)
                );
                assert_eq!(
                    updated_to.balance,
                    <tests::MockConfig as Config>::Funds::from(80u128)
                );
            }
        }

        mod failure {
            use crate::{
                block::Header,
                extrinsics::{self, SignedTransaction},
                types,
            };

            use super::*;

            #[test]
            fn test_execute_block_with_insufficient_balance() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                // Create account with insufficient balance
                let acc_alice: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [0; 32],
                        balance: 100,
                    });

                let acc_dave: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [1; 32],
                        balance: 50,
                    });

                // Create a block with a transfer transaction
                let transaction: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::Transfer {
                        from: [0; 32],
                        to: [1; 32],
                        amount: 150,
                    });

                let mut block = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                block.add_extrinsic(acc_alice).unwrap();
                block.add_extrinsic(acc_dave).unwrap();
                block.add_extrinsic(transaction).unwrap();

                assert!(stf.execute_block(block).is_ok());

                // Check that balances remain unchanged
                let updated_from: Account<MockConfig> = stf
                    .plugin
                    .get(
                        StoragePrefix::Account,
                        <tests::MockConfig as Config>::Hash::from([0; 32]),
                    )
                    .unwrap();
                let updated_to: Account<MockConfig> = stf
                    .plugin
                    .get(
                        StoragePrefix::Account,
                        <tests::MockConfig as Config>::Hash::from([1; 32]),
                    )
                    .unwrap();
                assert_eq!(updated_from.balance, 100);
                assert_eq!(updated_to.balance, 50);
            }
        }
    }

    mod validate_account {
        use super::*;

        mod success {
            use super::*;

            #[test]
            fn test_validate_new_account() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                let new_account = Account {
                    account_id: [0; 32],
                    balance: 100,
                };

                stf.plugin
                    .set(
                        StoragePrefix::Account,
                        &new_account.account_id,
                        &new_account.balance,
                    )
                    .map_err(StfError::Storage)
                    .unwrap();

                assert!(stf.validate_account(new_account).is_ok());
            }
        }

        mod failure {
            use crate::{
                block::Header,
                extrinsics::{self, SignedTransaction},
                types,
            };

            use super::*;

            #[test]
            fn test_validate_existing_account() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                let acc_alice: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [0; 32],
                        balance: 100,
                    });

                let mut block = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                block.add_extrinsic(acc_alice).unwrap();

                assert!(stf.execute_block(block).is_ok());

                let same_alice_account = Account {
                    account_id: [0; 32],
                    balance: 100,
                };

                // This fails because the account already exists...
                assert!(stf.validate_account(same_alice_account).is_err());
            }
        }
    }

    mod get_block_hash {
        use super::*;

        mod success {
            use crate::block::Header;

            use super::*;

            #[test]
            fn test_get_existing_block_hash() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                let block: Block<MockConfig> = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                stf.plugin
                    .set(StoragePrefix::Block, &1u64, &block.hash())
                    .unwrap();

                assert_eq!(stf.get_block_hash(Height::from(1)).unwrap(), block.hash());
            }
        }

        mod failure {
            use super::*;

            #[test]
            fn test_get_nonexistent_block_hash() {
                let plugin = Plugin::new();
                let stf = SimpleStf::<MockConfig>::new(plugin);

                assert!(stf.get_block_hash(Height::from(1)).is_err());
            }
        }
    }

    mod get_account {
        use super::*;

        mod success {
            use crate::{
                block::Header,
                extrinsics::{self, SignedTransaction},
                types,
            };

            use super::*;

            #[test]
            fn test_get_existing_account() {
                let plugin = Plugin::new();
                let mut stf = SimpleStf::<MockConfig>::new(plugin);

                let acc_alice: SignedTransaction<MockConfig> =
                    extrinsics::SignedTransaction::new(types::TransactionType::AccountCreation {
                        account_id: [0; 32],
                        balance: 100,
                    });

                let mut block = Block {
                    header: Header {
                        block_height: Height::from(1),
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                block.add_extrinsic(acc_alice).unwrap();

                assert!(stf.execute_block(block).is_ok());

                let retrieved_account = stf
                    .get_account(<tests::MockConfig as Config>::Hash::from([0; 32]))
                    .unwrap();
                assert_eq!(retrieved_account.account_id, [0; 32]);
                assert_eq!(retrieved_account.balance, 100);
            }
        }

        mod failure {
            use super::*;

            #[test]
            fn test_get_nonexistent_account() {
                let plugin = Plugin::new();
                let stf = SimpleStf::<MockConfig>::new(plugin);

                assert!(stf
                    .get_account(<tests::MockConfig as Config>::Hash::default())
                    .is_err());
            }
        }
    }
}
