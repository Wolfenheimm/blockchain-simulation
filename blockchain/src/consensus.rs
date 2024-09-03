use crate::{block::Block, extrinsics::SignedTransaction, Config};

/// A simulated network of nodes that can send blocks to other nodes.
trait Nodes<T: Config> {
    /// Return the block with the given number.
    ///
    /// Should be used when a node encounters a new canonical fork and needs to reorg to the new chain.
    fn request_block(&self, block_number: T::Height) -> Block;
}

trait Consensus<T: Config> {
    /// The simulated network of nodes from which you can request blocks.
    type NodeNetwork: Nodes<T>;

    /// Import and validate a new block and execute the STF.
    ///
    /// The consensus protocol should assume the highest block is the best block (i.e. the canonical chain).
    /// Since this is a simplified example of a consensus protocol, when a reorg happens, we can call into the [`Nodes`]
    /// to request every block number we don't have.
    fn import_block(&self, block: Block);
}

pub struct Node {
    pub transaction_pool: Vec<SignedTransaction>,
}

impl<T: Config> Nodes<T> for Node {
    fn request_block(&self, block_number: T::Height) -> Block {
        todo!()
    }
}

pub trait NodeTrait {
    fn add_transaction(&mut self, transaction: SignedTransaction);
}

impl NodeTrait for Node {
    fn add_transaction(&mut self, transaction: SignedTransaction) {
        self.transaction_pool.push(transaction);
    }
}
