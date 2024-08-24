#[derive(Debug, Clone, Copy)]
struct Account {
    hash: [u8; 32],
    balance: u128,
}

pub type AccountId = u64;
