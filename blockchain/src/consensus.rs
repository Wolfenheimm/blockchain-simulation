use crate::Config;

/// A simulated network of nodes that can send blocks to other nodes.
trait Nodes<T: Config> {
    /// Returns all the blocks after the given block hash.
    ///
    /// Should be used when a node encounters a new canonical fork and needs to reorg to the new chain.
    fn sync_blocks_from(&self, block_hash: Hash);
}

trait Consensus<T: Config> {
    type NodeNetwork: Nodes<T>;

    /// Import and validate a new block and execute the STF.
    ///
    /// The consensus protocol should assume the highest block is the best block (i.e. the canonical chain).
    /// Since this is a simplified example of a consensus protocol, when a reorg happens, we can call into the [`Nodes`]
    /// to ask for all the blocks from the last finalized block known to the node.
    fn import_block(&self, block: T::Block);
}
