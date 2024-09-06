pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use account::Account;
use block::{Block, BlockTrait, Header};
use consensus::{Consensus, ConsensusT, Node, NodeTrait};
use extrinsics::SignedTransaction;
use lazy_static::lazy_static;
use rand::Rng;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use types::{MaxBlockHeightImpl, MaxBlockWeightImpl};

pub trait Config {
    type MaxBlockWeight: Get<u64>;
    type MaxBlockHeight: Get<u64>;
}

pub trait Get<T> {
    fn get(&self) -> T;
}

impl Get<u64> for MaxBlockWeightImpl {
    fn get(&self) -> u64 {
        10
    }
}

impl Get<u64> for MaxBlockHeightImpl {
    fn get(&self) -> u64 {
        100000
    }
}

#[derive(Debug)]
struct MainNetConfig;

impl Config for MainNetConfig {
    type MaxBlockWeight = MaxBlockWeightImpl;
    type MaxBlockHeight = MaxBlockHeightImpl;
}

lazy_static! {
    /// The genesis block: the first block in the blockchain.
    static ref GENESIS_BLOCK: Block = Block {
        header: Header {
            block_height: 0,
            parent_hash: [0;32],
            state_root: [0;32],
            extrinsics_root: [0;32],
            block_weight: 0,
        },
        extrinsics: vec![].into(),
    };

    static ref ALICE: Account = Account {
        account_id: [0;32],
        balance: 1000000000,
    };

    static ref DAVE: Account = Account {
        account_id: [1;32],
        balance: 1000,
    };
}

fn main() {
    // TODO: Simulate the blockchain in its totality
    let mut block_height = 0;
    let plugin = plugin::Plugin::new();
    let node = Arc::new(Mutex::new(Node {
        transaction_pool: vec![].into(),
    }));
    let consensus = Arc::new(Consensus {
        node_network: Arc::clone(&node), // Here, the node itself serves as the node network
        phantom: std::marker::PhantomData::<MainNetConfig>,
    });
    let mut stf: stf::SimpleStf<MainNetConfig> = stf::SimpleStf::new(MainNetConfig, plugin);

    println!("BLOCKCHAIN BEGIN ~>");

    thread::scope(|s| {
        let node_clone = Arc::clone(&node);
        s.spawn(move || loop {
            {
                let mut node = node_clone.lock().unwrap();
                let num: u32 = rand::thread_rng().gen_range(0..=2);

                match num {
                    0 => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Transfer {
                            from: ALICE.account_id,
                            to: DAVE.account_id,
                            amount: 100,
                        },
                    )),
                    1 => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Mint {
                            to: DAVE.account_id,
                            amount: 100,
                        },
                    )),
                    2 => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            from: ALICE.account_id,
                            amount: 100,
                        },
                    )),
                    _default => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            from: ALICE.account_id,
                            amount: 100,
                        },
                    )),
                }
            }
            thread::sleep(Duration::from_millis(400));
        });
        s.spawn(move || loop {
            let mut block = block::Block {
                header: Header {
                    block_height: block_height,
                    parent_hash: GENESIS_BLOCK.hash(),
                    state_root: [0; 32],
                    extrinsics_root: [0; 32],
                    block_weight: 0,
                },
                extrinsics: Vec::new(),
            };

            if block_height != 0 {
                let mut node = node.lock().unwrap();

                // Keep pulling from the transaction pool until the block weight limit is reached
                while let Some(transaction) = node.transaction_pool.pop_back() {
                    // Check if the extrinsic can be added
                    match block.add_extrinsic(transaction) {
                        Ok(_) => {}
                        Err(e) => {
                            // Block weight limit exceeded, break the loop and rollback...
                            println!("{}", e);
                            node.transaction_pool.push_back(transaction);
                            break;
                        }
                    }
                }
            }

            // Import the block with the collected transactions and final weight
            consensus.import_block(&mut block, &mut stf);

            // Increment block height for the next block
            block_height += 1;

            // Sleep for a while before producing the next block
            thread::sleep(Duration::from_millis(6000));
        });
    });
}
