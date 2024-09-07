pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use crate::block::BlockTrait;
use block::Header;
use consensus::{Consensus, ConsensusT, Node, RpcNode};
use rand::Rng;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};
use std::ops::{Add, AddAssign, Sub};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
use types::{Height, MaxBlockHeight, MaxBlockWeight};

pub trait Config {
    type MaxBlockWeight: Get<Self::WeightType>;
    type MaxBlockHeight: Get<Self::HeightType>;
    type WeightType: Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Add<Output = Self::WeightType>
        + From<u64>
        + AddAssign
        + PartialOrd
        + Display;
    type HeightType: Clone
        + Serialize
        + DeserializeOwned
        + Debug
        + Display
        + PartialEq
        + From<u64>
        + Sub<Output = Self::HeightType>
        + Into<Vec<u8>>
        + Zero
        + One
        + AddAssign;
    type Hash: Serialize
        + DeserializeOwned
        + Debug
        + AsRef<[u8]>
        + Copy
        + PartialEq
        + From<[u8; 32]>;
    type Funds: Copy
        + Debug
        + Serialize
        + DeserializeOwned
        + From<u128>
        + PartialOrd
        + Add<Output = Self::Funds>
        + Sub<Output = Self::Funds>;
}

pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

pub trait Get<T> {
    fn get() -> T;
}

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
    // TODO: Simulate the blockchain in its totality
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

                // Keep pulling from the transaction pool until the block weight limit is reached
                //let x = node.transaction_pool.get(node.transaction_pool.len());
                //let y = node.transaction_pool.split_off(at);
                while let Some(transaction) = node.transaction_pool.pop_back() {
                    // Check if the extrinsic can be added
                    match block.add_extrinsic(transaction.clone()) {
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
            block_height += Height::one();

            // Sleep for a while before producing the next block
            thread::sleep(Duration::from_millis(6000));
        });
    });
}
