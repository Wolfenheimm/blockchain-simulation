use crate::account::AccountId;

pub type Extrinsics = Vec<Extrinsic>;

#[derive(Debug, Clone)]
pub struct Extrinsic {
    pub call: Call,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub from: AccountId,
    pub to: AccountId,
    pub amount: u128,
}
