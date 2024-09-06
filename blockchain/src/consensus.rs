use crate::stf::Stf;
use crate::{extrinsics, stf, types};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{block::Block, extrinsics::SignedTransaction, Config};

/// A simulated network of nodes that can send blocks to other nodes.
pub trait Nodes<T: Config> {
    /// Return the block with the given number.
    ///
    /// Should be used when a node encounters a new canonical fork and needs to reorg to the new chain.
    fn request_block(&self, block_number: T::MaxBlockHeight) -> Block;
}

pub trait ConsensusT<T: Config> {
    /// Import and validate a new block and execute the STF.
    ///
    /// The consensus protocol should assume the highest block is the best block (i.e. the canonical chain).
    /// Since this is a simplified example of a consensus protocol, when a reorg happens, we can call into the [`Nodes`]
    /// to request every block number we don't have.
    fn import_block(&self, block: &mut Block, stf: &mut stf::SimpleStf<T>);
}

#[derive(Debug)]
pub struct Consensus<T: Config, N: Nodes<T>> {
    pub node_network: N,
    pub phantom: std::marker::PhantomData<T>,
}

impl<T: Config, N: Nodes<T>> ConsensusT<T> for Consensus<T, N> {
    fn import_block(&self, block: &mut Block, stf: &mut stf::SimpleStf<T>) {
        if block.header.block_height == 0 {
            // Genesis block
            block.extrinsics.push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    account_id: [0; 32], // ALICE
                    balance: 10000000000,
                },
            ));
            block.extrinsics.push(extrinsics::SignedTransaction::new(
                types::TransactionType::AccountCreation {
                    account_id: [1; 32], // DAVE
                    balance: 1000,
                },
            ));
            stf.execute_block(block.clone());
        } else {
            println!("Block Height: {}", block.header.block_height);
            block.header.parent_hash = stf.get_block_hash(block.header.block_height - 1).unwrap();
            match stf.validate_block(block.clone()) {
                Ok(_) => {
                    // Execute the block
                    stf.execute_block(block.clone());
                }
                Err(e) => {
                    println!("Error: {}", e);
                }
            }

            println!("Account ALICE: {:?}", stf.get_account([0; 32]));
            println!("Account DAVE: {:?}", stf.get_account([1; 32]));
        }
    }
}

#[derive(Debug)]
pub struct Node {
    pub transaction_pool: VecDeque<SignedTransaction>,
}

impl<T: Config> Nodes<T> for Arc<Mutex<Node>> {
    fn request_block(&self, block_number: T::MaxBlockHeight) -> Block {
        todo!()
    }
}

pub trait NodeTrait {
    fn add_transaction(&mut self, transaction: SignedTransaction);
    fn get_transactions(&self) -> VecDeque<SignedTransaction>;
}

impl NodeTrait for Node {
    fn add_transaction(&mut self, transaction: SignedTransaction) {
        self.transaction_pool.push_front(transaction);
        println!("New -> {:?}", transaction)
    }
    fn get_transactions(&self) -> VecDeque<SignedTransaction> {
        self.transaction_pool.clone()
    }
}
