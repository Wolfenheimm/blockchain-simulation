pub mod account;
pub mod block;
pub mod consensus;
pub mod extrinsics;
pub mod plugin;
pub mod state;
pub mod stf;
pub mod types;
use crate::stf::Stf;
use account::Account;
use block::{Block, BlockTrait, Header};
use lazy_static::lazy_static;
use plugin::StoragePlugin;
use stf::StoragePrefix;
use types::BlockHeightTrait;

pub trait Config {
    const MAX_BLOCK_WEIGHT: u64;
    type Height: BlockHeightTrait;
}

struct MainNetConfig;

impl Config for MainNetConfig {
    const MAX_BLOCK_WEIGHT: u64 = 1000;
    type Height = u64;
}

lazy_static! {
    /// The genesis block: the first block in the blockchain.
    static ref GENESIS_BLOCK: Block = Block {
        header: Header {
            block_height: 0,
            parent_hash: [0;32],
            state_root: [0;32],
            extrinsics_root: [0;32],
        },
        extrinsics: vec![],
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
    // TODO: Simulate the blockchain
    let mut blockchain: Vec<Block> = vec![GENESIS_BLOCK.clone()];
    let mut plugin = plugin::Plugin::new();
    let stf = stf::SimpleStf::new(MainNetConfig);

    // Add the genesis block to the state
    stf.execute_block(GENESIS_BLOCK.clone(), &mut plugin);

    for i in 1..=10 {
        let new_block = Block {
            header: Header {
                block_height: i,
                parent_hash: blockchain[(i - 1) as usize].hash(),
                state_root: [i.try_into().unwrap(); 32],
                extrinsics_root: [i.try_into().unwrap(); 32],
            },
            extrinsics: vec![],
        };

        // Validate the block
        match stf.validate_block(new_block.clone(), &mut plugin) {
            Ok(_) => {
                // Execute the block
                stf.execute_block(new_block.clone(), &mut plugin);
                blockchain.push(new_block);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // DEBUGGING
    // --Complete state printout
    plugin.get_state().print_state();
    // --Print a known block
    let test_height: u64 = 5;
    let block_hash: Option<[u8; 32]> = plugin.get(StoragePrefix::Block, test_height);
    let block: Option<Block> = plugin.get(StoragePrefix::Block, block_hash.unwrap());
    println!(
        "Block 5 -> Block Hash: {:?}, Block Data: {:?}",
        block_hash, block
    );
}
