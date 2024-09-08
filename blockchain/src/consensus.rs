use crate::stf::Stf;
use crate::types::ConsensusError;
use crate::{extrinsics, stf, types};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{block::Block, extrinsics::SignedTransaction, Config};

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
        if block.header.block_height == T::HeightType::from(0) {
            // Genesis block
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
            block.header.parent_hash = stf
                .get_block_hash(block.header.block_height.clone() - T::HeightType::from(1))
                .unwrap();
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

            println!(
                "Account ALICE: {:?}",
                stf.get_account(T::Hash::from([0; 32]))
            );
            println!(
                "Account DAVE: {:?}",
                stf.get_account(T::Hash::from([1; 32]))
            );
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
    fn request_block(&self, block_number: T::MaxBlockHeight) -> Block<T> {
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
