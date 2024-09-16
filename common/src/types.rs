use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::{
    // Add this line to import the Display trait
    fmt::{Debug, Display},
    ops::{AddAssign, Sub},
};
use thiserror::Error;

pub trait Config {
    type MaxBlockWeight: Get<Self::WeightType>;
    type MaxBlockHeight: Get<Self::HeightType>;
    type WeightType: Clone
        + Debug
        + Serialize
        + DeserializeOwned
        + Add<Output = Self::WeightType>
        + From<u64>
        + AddAssign
        + PartialOrd
        + Display;
    type HeightType: Clone
        + Serialize
        + DeserializeOwned
        + Debug
        + Display
        + PartialEq
        + From<u64>
        + Sub<Output = Self::HeightType>
        + Into<Vec<u8>>
        + Zero
        + One
        + AddAssign;
    type Hash: Serialize
        + DeserializeOwned
        + Debug
        + AsRef<[u8]>
        + Copy
        + PartialEq
        + From<[u8; 32]>
        + Default;
    type Funds: Copy
        + Debug
        + Serialize
        + DeserializeOwned
        + From<u128>
        + PartialOrd
        + Add<Output = Self::Funds>
        + Sub<Output = Self::Funds>;
}

pub struct MaxBlockHeight;
pub struct FundSum;
pub struct MaxBlockWeight;

impl Get<u64> for MaxBlockWeight {
    fn get() -> u64 {
        200
    }
}

impl Get<Height> for MaxBlockHeight {
    fn get() -> Height {
        Height::from(100000)
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub struct Height(pub u64);

impl Into<Vec<u8>> for Height {
    fn into(self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl From<u64> for Height {
    fn from(value: u64) -> Self {
        Height(value)
    }
}

impl Sub for Height {
    type Output = Height;

    fn sub(self, rhs: Self) -> Self::Output {
        Height(self.0 - rhs.0)
    }
}

impl Display for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Zero for Height {
    fn zero() -> Self {
        Height(0)
    }
}

impl One for Height {
    fn one() -> Self {
        Height(1)
    }
}

impl AddAssign for Height {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

pub trait Zero {
    fn zero() -> Self;
}

pub trait One {
    fn one() -> Self;
}

pub trait Get<T> {
    fn get() -> T;
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType<T>
where
    T: Config,
{
    Transfer {
        from: T::Hash,
        to: T::Hash,
        amount: T::Funds,
    },
    Mint {
        to: T::Hash,
        amount: T::Funds,
    },
    Burn {
        from: T::Hash,
        amount: T::Funds,
    },
    AccountCreation {
        account_id: T::Hash,
        balance: T::Funds,
    },
}

impl<T: Config> Clone for TransactionType<T> {
    fn clone(&self) -> Self {
        match &self {
            Self::Transfer { from, to, amount } => Self::Transfer {
                from: from.clone(),
                to: to.clone(),
                amount: amount.clone(),
            },
            Self::Mint { to, amount } => Self::Mint {
                to: to.clone(),
                amount: amount.clone(),
            },
            Self::Burn { from, amount } => Self::Burn {
                from: from.clone(),
                amount: amount.clone(),
            },
            Self::AccountCreation {
                account_id,
                balance,
            } => Self::AccountCreation {
                account_id: account_id.clone(),
                balance: balance.clone(),
            },
        }
    }
}

impl<T: Config> TransactionType<T> {
    pub fn weight(&self) -> T::WeightType {
        match self {
            Self::Transfer { .. } => T::WeightType::from(10),
            Self::Mint { .. } => T::WeightType::from(15),
            Self::Burn { .. } => T::WeightType::from(20),
            Self::AccountCreation { .. } => T::WeightType::from(7),
        }
    }
}

// TODO: Implement it, may have issues with current Option usage
#[derive(Debug)]
pub enum TransactionError<T: Config> {
    AccountNotFound(T::Hash),
    InsufficientBalance {
        account_id: T::Hash,
        balance: T::Funds,
        amount: T::Funds,
    },
}

#[derive(Debug, Clone, Error)]
pub enum StorageError {
    #[error("Serialization Error: {0}")]
    SerializationError(String),
    #[error("Deserialization Error: {0}")]
    DeserializationError(String),
    #[error("Key Creation Error: {0}")]
    KeyCreationError(String),
    #[error("Key not found for: {0}")]
    KeyNotFound(String),
    #[error("Could not create Full Key: {0}")]
    CreateFullKeyError(String),
    #[error("Data Insertion Error: {0}")]
    DataInsertionError(String),
    #[error("Storage operation failed: {0}")]
    OperationFailed(String),
    #[error("Data Not Found: {0}")]
    DataNotFound(String),
}

#[derive(Debug, Clone, Error)]
pub enum StfError {
    #[error("Failed to execute block: {0}")]
    BlockExecutionError(String),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

#[derive(Debug, Clone, Error)]
pub enum ConsensusError {
    #[error("Failed to import block: {0}")]
    ImportBlockError(String),
    #[error("Stf error: {0}")]
    Stf(#[from] StfError),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StoragePrefix {
    Account,
    Block,
    Extrinsic,
}

#[cfg(test)]
mod tests {
    use super::*;

    // Define a mock configuration struct
    #[derive(Debug, PartialEq)]
    struct MockConfig;

    // Implement the Config trait for MockConfig
    impl Config for MockConfig {
        type MaxBlockWeight = MaxBlockWeight;
        type MaxBlockHeight = MaxBlockHeight;
        type WeightType = u64;
        type HeightType = Height;
        type Hash = [u8; 32];
        type Funds = u128;
    }

    mod test_height {
        mod success {
            use crate::{types::Height, types::One, types::Zero};

            #[test]
            fn test_height_from_u64() {
                let height = Height::from(10u64);
                assert_eq!(height.0, 10);
            }

            #[test]
            fn test_height_into_vec_u8() {
                let height = Height(0x1234_5678_9ABC_DEF0);
                let bytes: Vec<u8> = height.into();
                assert_eq!(bytes, vec![0xF0, 0xDE, 0xBC, 0x9A, 0x78, 0x56, 0x34, 0x12]);
            }

            #[test]
            fn test_height_subtraction() {
                let h1 = Height(100);
                let h2 = Height(30);
                assert_eq!(h1 - h2, Height(70));
            }

            #[test]
            fn test_height_display() {
                let height = Height(12345);
                assert_eq!(format!("{}", height), "12345");
            }

            #[test]
            fn test_height_zero_and_one() {
                assert_eq!(Height::zero(), Height(0));
                assert_eq!(Height::one(), Height(1));
            }

            #[test]
            fn test_height_add_assign() {
                let mut height = Height(10);
                height += Height(5);
                assert_eq!(height, Height(15));
            }
        }
    }

    mod test_transaction_type {
        mod success {
            use crate::types::{tests::MockConfig, TransactionType};

            #[test]
            fn test_transaction_type_weight() {
                let transfer = TransactionType::<MockConfig>::Transfer {
                    from: [0; 32],
                    to: [1; 32],
                    amount: 100,
                };
                assert_eq!(transfer.weight(), 10);

                let mint = TransactionType::<MockConfig>::Mint {
                    to: [2; 32],
                    amount: 50,
                };
                assert_eq!(mint.weight(), 15);

                let burn = TransactionType::<MockConfig>::Burn {
                    from: [3; 32],
                    amount: 25,
                };
                assert_eq!(burn.weight(), 20);

                let account_creation = TransactionType::<MockConfig>::AccountCreation {
                    account_id: [4; 32],
                    balance: 1000,
                };
                assert_eq!(account_creation.weight(), 7);
            }

            #[test]
            fn test_transaction_type_clone() {
                let transfer = TransactionType::<MockConfig>::Transfer {
                    from: [0; 32],
                    to: [1; 32],
                    amount: 100,
                };
                let cloned_transfer = transfer.clone();
                assert_eq!(transfer, cloned_transfer);
            }
        }
    }
}
