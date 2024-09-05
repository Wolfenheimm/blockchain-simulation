pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use rand::Rng;
use std::{
    default,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crate::stf::Stf;
use account::Account;
use block::{Block, BlockTrait, Header};
use consensus::{Consensus, ConsensusT, Node, NodeTrait};
use extrinsics::SignedTransaction;
use lazy_static::lazy_static;
use plugin::StoragePlugin;
use stf::StoragePrefix;
use types::{BlockHeightTrait, TransactionType};

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Height: BlockHeightTrait;
}

#[derive(Debug)]
struct MainNetConfig;

impl Config for MainNetConfig {
    const MAX_BLOCK_WEIGHT: u64 = 10;
    type Height = u64;
}

#[derive(Debug)]
struct Nodes;

lazy_static! {
    /// The genesis block: the first block in the blockchain.
    static ref GENESIS_BLOCK: Block = Block {
        header: Header {
            block_height: 0,
            parent_hash: [0;32],
            state_root: [0;32],
            extrinsics_root: [0;32],
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
                            weight: 1,
                            from: ALICE.account_id,
                            to: DAVE.account_id,
                            amount: 100,
                        },
                    )),
                    1 => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Mint {
                            weight: 1,
                            to: DAVE.account_id,
                            amount: 100,
                        },
                    )),
                    2 => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            weight: 1,
                            from: ALICE.account_id,
                            amount: 100,
                        },
                    )),
                    _default => node.add_transaction(extrinsics::SignedTransaction::new(
                        types::TransactionType::Burn {
                            weight: 1,
                            from: ALICE.account_id,
                            amount: 100,
                        },
                    )),
                }
            }
            thread::sleep(Duration::from_millis(1000));
        });
        s.spawn(move || loop {
            let mut current_transactions: Vec<SignedTransaction> = Default::default();

            if block_height != 0 {
                for _i in 1..=MainNetConfig::MAX_BLOCK_WEIGHT {
                    if node.lock().unwrap().transaction_pool.len() > 0 {
                        current_transactions
                            .push(node.lock().unwrap().transaction_pool.pop_back().unwrap());
                    } else {
                        break;
                    }
                }
            }

            consensus.import_block(
                &mut block::Block {
                    header: Header {
                        block_height: block_height,
                        parent_hash: GENESIS_BLOCK.hash(),
                        state_root: [0; 32],
                        extrinsics_root: [0; 32],
                    },
                    extrinsics: current_transactions.clone(),
                },
                &mut stf,
            );

            block_height += 1;

            thread::sleep(Duration::from_millis(6000));
        });
    });
}
