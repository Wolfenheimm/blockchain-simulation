use serde::Serialize;

use crate::types::TransactionType;

pub trait Extrinsics {
    type Extrinsic;

    fn add_extrinsic(&mut self, transaction_type: TransactionType);
    fn get_latest_extrinsic(&self) -> &Self::Extrinsic;
}

impl Extrinsics for Vec<SignedTransaction> {
    type Extrinsic = SignedTransaction;

    fn add_extrinsic(&mut self, transaction_type: TransactionType) {
        let new_extrinsic = SignedTransaction::new(transaction_type);

        self.push(new_extrinsic);
    }

    fn get_latest_extrinsic(&self) -> &Self::Extrinsic {
        self.last()
            .expect("The extrinsics have not been initialized")
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct SignedTransaction {
    pub transaction_type: TransactionType,
}

impl SignedTransaction {
    pub fn new(transaction_type: TransactionType) -> Self {
        SignedTransaction { transaction_type }
    }
}
