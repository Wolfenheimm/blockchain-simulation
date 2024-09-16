use common::block;
use common::block::BlockTrait;
use common::block::Header;
use common::extrinsics;
use common::types;
use common::types::Config;
use common::types::One;
use common::types::Zero;
use node::{Consensus, ConsensusT, Node, RpcNode};
use rand::Rng;
use runtime::plugin;
use runtime::stf;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use types::{Height, MaxBlockHeight, MaxBlockWeight};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
struct MainNetConfig;

impl Config for MainNetConfig {
    type MaxBlockWeight = MaxBlockWeight;
    type MaxBlockHeight = MaxBlockHeight;
    type WeightType = u64;
    type HeightType = Height;
    type Hash = [u8; 32];
    type Funds = u128;
}

fn main() {
    let mut block_height = Height::zero();
    let plugin = plugin::Plugin::new();
    let node = Arc::new(Mutex::new(Node {
        transaction_pool: vec![].into(),
    }));
    let consensus = Arc::new(Consensus {
        node_network: Arc::clone(&node), // Here, the node itself serves as the node network
        phantom: std::marker::PhantomData::<MainNetConfig>,
    });
    let mut stf: stf::SimpleStf<MainNetConfig> = stf::SimpleStf::new(plugin);

    println!("BLOCKCHAIN BEGIN ~>");

    thread::scope(|s| {
        let node_clone = Arc::clone(&node);
        s.spawn(move || loop {
            {
                let mut node = node_clone.lock().unwrap();
                let num: u32 = rand::thread_rng().gen_range(0..=2);

                match num {
                    0 => node.submit_extrinsic(extrinsics::SignedTransaction::new(
                        types::TransactionType::Transfer {
                            from: [0; 32],
                            to: [1; 32],
                            amount: 100,
                        },
                    )),
                    1 => node.submit_extrinsic(extrinsics::SignedTransaction::new(
                        types::TransactionType::Mint {
                            to: [1; 32],
                            amount: 100,
                        },
                    )),
                    2 => node.submit_extrinsic(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            from: [0; 32],
                            amount: 100,
                        },
                    )),
                    _default => node.submit_extrinsic(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            from: [0; 32],
                            amount: 0,
                        },
                    )),
                }
            }
            thread::sleep(Duration::from_millis(400));
        });
        s.spawn(move || loop {
            let mut block = block::Block {
                header: Header {
                    block_height,
                    parent_hash: [0; 32],
                    state_root: [0; 32],
                    extrinsics_root: [0; 32],
                    block_weight: 0,
                },
                extrinsics: Vec::new(),
            };

            // The genesis block shouldn't contain transactions other than the ones
            // currently hard-coded @ startup
            if block_height != Height::zero() {
                let mut node = node.lock().unwrap();

                // Keep pulling from the transaction pool until the block weight limit is reached
                while let Some(transaction) = node.transaction_pool.back() {
                    // Check if the extrinsic can be added
                    if block.can_add_extrinsic(transaction.weight()) {
                        block.add_extrinsic(transaction.clone()).unwrap();
                        node.transaction_pool.pop_back();
                    } else {
                        break;
                    }
                }
            }

            // Import the block with the collected transactions and final weight
            consensus.import_block(&mut block, &mut stf).unwrap();

            // Increment block height for the next block
            block_height += Height::one();

            // Sleep for a while before producing the next block
            thread::sleep(Duration::from_millis(6000));
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_blockchain_basic_operation {
        use super::*;

        mod success {
            use stf::Stf;

            use super::*;

            #[test]
            fn test_basic_blockchain_functionality() {
                let plugin = plugin::Plugin::new();
                let node = Arc::new(Mutex::new(Node {
                    transaction_pool: vec![].into(),
                }));
                let consensus = Arc::new(Consensus {
                    node_network: Arc::clone(&node),
                    phantom: std::marker::PhantomData::<MainNetConfig>,
                });
                let mut stf: stf::SimpleStf<MainNetConfig> = stf::SimpleStf::new(plugin);

                // Simulate blockchain operation for a few blocks
                let mut block_height = Height::zero();
                for _ in 0..5 {
                    let mut block = block::Block {
                        header: Header {
                            block_height,
                            parent_hash: [0; 32],
                            state_root: [0; 32],
                            extrinsics_root: [0; 32],
                            block_weight: 0,
                        },
                        extrinsics: Vec::new(),
                    };

                    if block_height != Height::zero() {
                        // Add some transactions to the pool
                        let mut node = node.lock().unwrap();
                        node.submit_extrinsic(extrinsics::SignedTransaction::new(
                            types::TransactionType::Transfer {
                                from: [0; 32],
                                to: [1; 32],
                                amount: 100,
                            },
                        ));
                        node.submit_extrinsic(extrinsics::SignedTransaction::new(
                            types::TransactionType::Mint {
                                to: [1; 32],
                                amount: 50,
                            },
                        ));

                        // Process transactions
                        while let Some(transaction) = node.transaction_pool.pop_back() {
                            // Check if the extrinsic can be added
                            if block.can_add_extrinsic(transaction.weight()) {
                                block.add_extrinsic(transaction.clone()).unwrap();
                            } else {
                                break;
                            }
                        }
                    }

                    // Import the block
                    consensus.import_block(&mut block, &mut stf).unwrap();

                    // Increment block height
                    block_height += Height::one();
                }

                // Verify final state
                let alice_account = stf.get_account([0; 32]).unwrap();
                let dave_account = stf.get_account([1; 32]).unwrap();

                assert!(
                    alice_account.balance < 10000000000,
                    "Alice's balance should have decreased"
                );
                assert!(
                    dave_account.balance > 1000,
                    "Dave's balance should have increased"
                );
            }
        }
    }

    mod test_blockchain_stress {
        use super::*;

        mod success {
            use stf::Stf;

            use super::*;

            #[test]
            fn test_high_transaction_volume() {
                let plugin = plugin::Plugin::new();
                let node = Arc::new(Mutex::new(Node {
                    transaction_pool: vec![].into(),
                }));
                let consensus = Arc::new(Consensus {
                    node_network: Arc::clone(&node),
                    phantom: std::marker::PhantomData::<MainNetConfig>,
                });
                let mut stf: stf::SimpleStf<MainNetConfig> = stf::SimpleStf::new(plugin);

                // Simulate high transaction volume
                let mut block_height = Height::zero();
                for _ in 0..10 {
                    let mut block = block::Block {
                        header: Header {
                            block_height,
                            parent_hash: [0; 32],
                            state_root: [0; 32],
                            extrinsics_root: [0; 32],
                            block_weight: 0,
                        },
                        extrinsics: Vec::new(),
                    };

                    if block_height != Height::zero() {
                        let mut node = node.lock().unwrap();
                        for _ in 0..100 {
                            node.submit_extrinsic(extrinsics::SignedTransaction::new(
                                types::TransactionType::Transfer {
                                    from: [0; 32],
                                    to: [1; 32],
                                    amount: 1,
                                },
                            ));
                        }

                        while let Some(transaction) = node.transaction_pool.pop_back() {
                            if block.add_extrinsic(transaction.clone()).is_err() {
                                node.transaction_pool.push_back(transaction);
                                break;
                            }
                        }
                    }

                    consensus.import_block(&mut block, &mut stf).unwrap();
                    block_height += Height::one();
                }

                // Verify final state after stress test
                let alice_account = stf.get_account([0; 32]).unwrap();
                let dave_account = stf.get_account([1; 32]).unwrap();

                assert!(
                    alice_account.balance < 10000000000,
                    "Alice's balance should have decreased significantly"
                );
                assert!(
                    dave_account.balance > 1000,
                    "Dave's balance should have increased significantly"
                );
                assert!(
                    node.lock().unwrap().transaction_pool.len() > 0,
                    "Transaction pool should still have pending transactions"
                );
            }
        }
    }
}
