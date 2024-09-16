use blake2::{Blake2s256, Digest};
use serde::{Deserialize, Serialize};

use crate::{extrinsics::SignedTransaction, types::Config, types::Get};

#[derive(Debug, Serialize, Deserialize)]
pub struct Block<T: Config> {
    pub header: Header<T>,
    pub extrinsics: Vec<SignedTransaction<T>>,
}

impl<T: Config> Clone for Block<T> {
    fn clone(&self) -> Self {
        Self {
            header: self.header.clone(),
            extrinsics: self.extrinsics.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header<T: Config> {
    pub block_height: T::HeightType,
    pub parent_hash: T::Hash,
    pub state_root: T::Hash,
    pub extrinsics_root: T::Hash,
    pub block_weight: T::WeightType,
}

impl<T: Config> Clone for Header<T> {
    fn clone(&self) -> Self {
        Self {
            block_height: self.block_height.clone(),
            parent_hash: self.parent_hash.clone(),
            state_root: self.state_root.clone(),
            extrinsics_root: self.extrinsics_root.clone(),
            block_weight: self.block_weight.clone(),
        }
    }
}

pub trait BlockTrait<T: Config> {
    fn extrinsics(&self) -> &Vec<SignedTransaction<T>>;
    fn hash(&self) -> [u8; 32];
    fn add_extrinsic(&mut self, extrinsic: SignedTransaction<T>) -> Result<(), String>;
    fn can_add_extrinsic(&self, weight: T::WeightType) -> bool;
}

// Implement the BlockTrait for the Block struct
impl<T: Config> BlockTrait<T> for Block<T> {
    fn extrinsics(&self) -> &Vec<SignedTransaction<T>> {
        &self.extrinsics
    }

    fn hash(&self) -> [u8; 32] {
        let mut hasher = Blake2s256::new();
        hasher.update(Into::<Vec<u8>>::into(self.header.block_height.clone()));
        hasher.update(self.header.parent_hash);
        hasher.update(self.header.state_root);
        hasher.update(self.header.extrinsics_root);
        hasher
            .finalize()
            .try_into()
            .expect("This hash has an expected size of 32 bytes")
    }

    fn can_add_extrinsic(&self, weight: T::WeightType) -> bool {
        self.header.block_weight.clone() + weight <= T::MaxBlockWeight::get()
    }

    // Method to add an extrinsic if it does not exceed the maximum block weight
    fn add_extrinsic(&mut self, extrinsic: SignedTransaction<T>) -> Result<(), String> {
        if self.can_add_extrinsic(extrinsic.weight()) {
            self.extrinsics.push(extrinsic.clone());
            self.header.block_weight += extrinsic.weight();
            Ok(())
        } else {
            Err(format!(
                "Block weight exceeded. Max allowed: {}, Current: {}, New Extrinsic: {}",
                T::MaxBlockWeight::get(),
                self.header.block_weight,
                extrinsic.weight()
            ))
        }
    }
}
