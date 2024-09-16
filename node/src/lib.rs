use common::types::{self, StfError};
use common::types::{Config, ConsensusError};
use common::{block, extrinsics};
use runtime::stf::{self, Stf};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{block::Block, extrinsics::SignedTransaction};

/// A simulated network of nodes that can send blocks to other nodes.
pub trait Nodes<T: Config>
where
    T: Serialize + DeserializeOwned + Debug,
{
    /// Return the block with the given number.
    ///
    /// Should be used when a node encounters a new canonical fork and needs to reorg to the new chain.
    fn request_block(&self, block_number: T::MaxBlockHeight) -> Block<T>;
}

pub trait ConsensusT<T: Config>
where
    T: Serialize + DeserializeOwned + Debug,
{
    /// Import and validate a new block and execute the STF.
    ///
    /// The consensus protocol should assume the highest block is the best block (i.e. the canonical chain).
    /// Since this is a simplified example of a consensus protocol, when a reorg happens, we can call into the [`Nodes`]
    /// to request every block number we don't have.
    fn import_block(
        &self,
        block: &mut Block<T>,
        stf: &mut stf::SimpleStf<T>,
    ) -> Result<(), ConsensusError>;
}

#[derive(Debug)]
pub struct Consensus<T: Config, N: Nodes<T>>
where
    T: Serialize + DeserializeOwned + Debug,
{
    pub node_network: N,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T: Config, N: Nodes<T>> ConsensusT<T> for Consensus<T, N>
where
    T: Serialize + DeserializeOwned + Debug,
{
    fn import_block(
        &self,
        block: &mut Block<T>,
        stf: &mut stf::SimpleStf<T>,
    ) -> Result<(), ConsensusError> {
        // Here we inject test accounts into the genesis block
        if block.header.block_height == T::HeightType::from(0) {
            block.extrinsics.push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    account_id: T::Hash::from([0; 32]), // ALICE
                    balance: T::Funds::from(10000000000),
                },
            ));
            block.extrinsics.push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    account_id: T::Hash::from([1; 32]), // DAVE
                    balance: T::Funds::from(1000),
                },
            ));
            stf.execute_block(block.clone())
                .map_err(ConsensusError::Stf)?;
        } else {
            // Set the parent hash of the imported block
            block.header.parent_hash = stf
                .get_block_hash(block.header.block_height.clone() - T::HeightType::from(1))
                .map_err(|e| ConsensusError::Stf(StfError::Storage(e)))?;
            match stf.validate_block(block.clone()) {
                Ok(_) => {
                    // Execute the block
                    stf.execute_block(block.clone())
                        .map_err(ConsensusError::Stf)?;
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            // Debug
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
            println!(
                "Account ALICE: {:?}",
                stf.get_account(T::Hash::from([0; 32]))
            );
            println!(
                "Account DAVE: {:?}",
                stf.get_account(T::Hash::from([1; 32]))
            );
            println!("~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Node<T: Config> {
    pub transaction_pool: VecDeque<SignedTransaction<T>>,
}

impl<T: Config> Nodes<T> for Arc<Mutex<Node<T>>>
where
    T: Serialize + DeserializeOwned + Debug,
{
    fn request_block(&self, _block_number: T::MaxBlockHeight) -> Block<T> {
        todo!()
    }
}

pub trait RpcNode<T: Config> {
    fn submit_extrinsic(&mut self, transaction: SignedTransaction<T>);
    fn pending_extrinsics(&self) -> VecDeque<SignedTransaction<T>>;
}

impl<T: Config> RpcNode<T> for Node<T>
where
    T: Debug,
{
    fn submit_extrinsic(&mut self, transaction: SignedTransaction<T>) {
        self.transaction_pool.push_front(transaction.clone());
        println!("New -> {:?}", transaction)
    }
    fn pending_extrinsics(&self) -> VecDeque<SignedTransaction<T>> {
        self.transaction_pool.clone()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use types::{Height, MaxBlockHeight, MaxBlockWeight};

    use super::*;
    use crate::stf::SimpleStf;
    use crate::types::TransactionType;
    use std::sync::{Arc, Mutex};

    // Mock implementation of Config trait for testing
    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct MockConfig;
    impl Config for MockConfig {
        type MaxBlockWeight = MaxBlockWeight;
        type MaxBlockHeight = MaxBlockHeight;
        type WeightType = u64;
        type HeightType = Height;
        type Hash = [u8; 32];
        type Funds = u128;
    }

    mod test_import_block {
        use super::*;

        mod success {
            use crate::{
                block::{self, BlockTrait, Header},
                types::{One, Zero},
            };

            use super::*;

            #[test]
            fn test_import_genesis_block() {
                let block_height = Height::zero();

                let node = Arc::new(Mutex::new(Node {
                    transaction_pool: vec![].into(),
                }));

                let consensus = Arc::new(Consensus {
                    node_network: Arc::clone(&node), // Here, the node itself serves as the node network
                    phantom: std::marker::PhantomData::<MockConfig>,
                });

                let mut stf = SimpleStf::new(runtime::plugin::Plugin::new());
                let mut genesis_block = block::Block {
                    header: Header {
                        block_height,
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                assert!(consensus.import_block(&mut genesis_block, &mut stf).is_ok());

                // Because the genesis block is hardcoded to create accounts we will use these
                // to test functionality
                assert!(stf.get_account([0; 32]).is_ok());
                assert!(stf.get_account([1; 32]).is_ok());
            }

            #[test]
            fn test_import_regular_block() {
                let mut block_height = Height::zero();

                let node = Arc::new(Mutex::new(Node {
                    transaction_pool: vec![].into(),
                }));

                let consensus = Arc::new(Consensus {
                    node_network: Arc::clone(&node), // Here, the node itself serves as the node network
                    phantom: std::marker::PhantomData::<MockConfig>,
                });
                let mut stf = SimpleStf::new(runtime::plugin::Plugin::new());

                // Import genesis block first
                let mut genesis_block = block::Block {
                    header: Header {
                        block_height,
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };
                consensus
                    .import_block(&mut genesis_block, &mut stf)
                    .unwrap();

                // Now import a regular block
                block_height += Height::one();
                let mut regular_block = block::Block {
                    header: Header {
                        block_height,
                        parent_hash: genesis_block.hash(),
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };
                assert!(consensus.import_block(&mut regular_block, &mut stf).is_ok());
            }
        }

        mod failure {
            use crate::{
                block::{self, Header},
                types::{One, Zero},
            };

            use super::*;

            #[test]
            fn test_import_block_with_invalid_height() {
                let mut block_height = Height::zero();

                let node = Arc::new(Mutex::new(Node {
                    transaction_pool: vec![].into(),
                }));

                let consensus = Arc::new(Consensus {
                    node_network: Arc::clone(&node), // Here, the node itself serves as the node network
                    phantom: std::marker::PhantomData::<MockConfig>,
                });
                let mut stf = SimpleStf::new(runtime::plugin::Plugin::new());

                // We need a prior block before knowing if the parent hash is invalid...
                let mut genesis_block: Block<MockConfig> = block::Block {
                    header: Header {
                        block_height,
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };

                consensus
                    .import_block(&mut genesis_block, &mut stf)
                    .unwrap();

                block_height += Height::one();
                block_height += Height::one();

                // Try to import a block with an invalid height
                let mut invalid_block = block::Block {
                    header: Header {
                        block_height,
                        parent_hash: [0; 32],
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                        block_weight: 0,
                    },
                    extrinsics: Vec::new(),
                };
                assert!(consensus
                    .import_block(&mut invalid_block, &mut stf)
                    .is_err());
            }
        }
    }

    mod test_submit_extrinsic {
        use super::*;

        mod success {
            use super::*;

            #[test]
            fn test_submit_single_extrinsic() {
                let mut node = Node::<MockConfig> {
                    transaction_pool: VecDeque::new(),
                };

                let transaction = SignedTransaction::new(TransactionType::Transfer {
                    from: [0; 32],
                    to: [1; 32],
                    amount: 100,
                });

                node.submit_extrinsic(transaction.clone());

                assert_eq!(node.transaction_pool.len(), 1);
                assert_eq!(node.transaction_pool[0], transaction);
            }

            #[test]
            fn test_submit_multiple_extrinsics() {
                let mut node = Node::<MockConfig> {
                    transaction_pool: VecDeque::new(),
                };

                let transaction1 = SignedTransaction::new(TransactionType::Transfer {
                    from: [0; 32],
                    to: [1; 32],
                    amount: 100,
                });
                let transaction2 = SignedTransaction::new(TransactionType::Transfer {
                    from: [1; 32],
                    to: [0; 32],
                    amount: 50,
                });

                node.submit_extrinsic(transaction1.clone());
                node.submit_extrinsic(transaction2.clone());

                assert_eq!(node.transaction_pool.len(), 2);
                assert_eq!(node.transaction_pool[0], transaction2);
                assert_eq!(node.transaction_pool[1], transaction1);
            }
        }

        mod failure {
            // There are no failure cases for submit_extrinsic as it always succeeds
        }
    }

    mod test_pending_extrinsics {
        use super::*;

        mod success {
            use super::*;

            #[test]
            fn test_pending_extrinsics_empty() {
                let node = Node::<MockConfig> {
                    transaction_pool: VecDeque::new(),
                };

                assert!(node.pending_extrinsics().is_empty());
            }

            #[test]
            fn test_pending_extrinsics_with_transactions() {
                let mut node = Node::<MockConfig> {
                    transaction_pool: VecDeque::new(),
                };

                let transaction1 = SignedTransaction::new(TransactionType::Transfer {
                    from: [0; 32],
                    to: [1; 32],
                    amount: 100,
                });
                let transaction2 = SignedTransaction::new(TransactionType::Transfer {
                    from: [1; 32],
                    to: [0; 32],
                    amount: 50,
                });

                node.submit_extrinsic(transaction1.clone());
                node.submit_extrinsic(transaction2.clone());

                let pending = node.pending_extrinsics();
                assert_eq!(pending.len(), 2);
                assert_eq!(pending[0], transaction2);
                assert_eq!(pending[1], transaction1);
            }
        }
    }
}
