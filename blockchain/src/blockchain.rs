use std::collections::VecDeque;

pub trait Blockchain<E> {
    type Block;

    fn get_latest_block(&self) -> &Self::Block;

    fn add_block(&mut self, block: Self::Block);
}

#[derive(Debug, Clone)]
pub struct Block<E> {
    pub hash: [u8; 32],
    pub parent_hash: [u8; 32],
    pub state_root: [u8; 32],
    pub extrinsics: E, // Generic E allows for different transaction types...
}

pub struct SimpleBlockchain<E> {
    pub blocks: VecDeque<Block<E>>,
}

impl<E> SimpleBlockchain<E> {
    pub fn new(genesis_block: Block<E>) -> Self {
        let mut blocks = VecDeque::new();
        blocks.push_back(genesis_block);
        Self { blocks }
    }
}

impl<E> Blockchain<E> for SimpleBlockchain<E> {
    type Block = Block<E>;

    fn get_latest_block(&self) -> &Self::Block {
        self.blocks.back().unwrap()
    }

    fn add_block(&mut self, block: Self::Block) {
        self.blocks.push_back(block);
    }
}
