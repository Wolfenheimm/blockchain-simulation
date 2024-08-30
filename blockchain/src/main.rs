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
    // TODO: Simulate the blockchain in its totality
    let mut blockchain: Vec<Block> = vec![GENESIS_BLOCK.clone()];
    let mut plugin = plugin::Plugin::new();
    let stf: stf::SimpleStf<MainNetConfig> = stf::SimpleStf::new(MainNetConfig);

    // Add the genesis block to the state
    stf.execute_block(GENESIS_BLOCK.clone(), &mut plugin);

    for i in 1..=10 {
        let mut new_block = Block {
            header: Header {
                block_height: i,
                parent_hash: blockchain[(i - 1) as usize].hash(),
                state_root: [0; 32], // Placeholder -> Execute block should update this at the end once everything is done
                extrinsics_root: [0; 32], // Placeholder -> Execute block should update this at the end once everything is done
            },
            extrinsics: vec![], // TODO: Extrinsics Pool
        };

        new_block
            .extrinsics
            .push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    weight: 1,
                    account_id: ALICE.account_id,
                    balance: 10000000,
                },
            ));

        new_block
            .extrinsics
            .push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    weight: 1,
                    account_id: DAVE.account_id,
                    balance: 1000,
                },
            ));

        // TODO: Add extrinsics validation -> don't use push, use a function
        // TODO: use while loop to add extrinsics, clause to stop at MAX_BLOCK_WEIGHT
        // STF to save to state for <extrinsics_root, extrinsics>
        // Technically transfer 5k to dave
        for _i in 1..=5 {
            new_block
                .extrinsics
                .push(extrinsics::SignedTransaction::new(
                    types::TransactionType::Transfer {
                        weight: 1,
                        from: ALICE.account_id,
                        to: DAVE.account_id,
                        amount: 100,
                    },
                ));
        }

        // Technically mint 100 to alice
        new_block
            .extrinsics
            .push(extrinsics::SignedTransaction::new(
                types::TransactionType::Mint {
                    weight: 1,
                    to: ALICE.account_id,
                    amount: 10,
                },
            ));

        // Technically burn 10k from alice
        new_block
            .extrinsics
            .push(extrinsics::SignedTransaction::new(
                types::TransactionType::Burn {
                    weight: 1,
                    from: ALICE.account_id,
                    amount: 1000,
                },
            ));

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
    //plugin.get_state().print_state();
    // --Print a known block
    let test_height: u64 = 5;
    let block_hash: Option<[u8; 32]> = plugin.get(StoragePrefix::Block, test_height);
    let block: Option<Block> = plugin.get(StoragePrefix::Block, block_hash.unwrap_or_default());
    println!(
        "Example: Block {} ->\nBlock Hash: {:?}\nBlock Data: {:?}",
        test_height,
        block_hash.unwrap_or_default(),
        block.unwrap_or_default()
    );
    let alice: Option<Account> = plugin.get(StoragePrefix::Account, ALICE.account_id);
    let dave: Option<Account> = plugin.get(StoragePrefix::Account, DAVE.account_id);
    println!("Alice: {:?}", alice);
    println!("Dave: {:?}", dave);
}
