use crate::Get;
use crate::{Config, One, Zero};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Formatter};
use std::{
    // Add this line to import the Display trait
    fmt::{Debug, Display},
    ops::{AddAssign, Sub},
};

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

#[derive(Debug, Clone)]
pub enum StorageError {
    SerializationError(String),
    DeserializationError(String),
    KeyCreationError(String),
    KeyNotFound(String),
    CreateFullKeyError(String),
    DataNotFound,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::SerializationError(msg) => write!(f, "Serialization Error: {}", msg),
            StorageError::DeserializationError(msg) => write!(f, "Deserialization Error: {}", msg),
            StorageError::KeyCreationError(msg) => write!(f, "Key Creation Error: {}", msg),
            StorageError::KeyNotFound(msg) => write!(f, "Key not found for: {}", msg),
            StorageError::CreateFullKeyError(msg) => {
                write!(f, "Could not create Full Key: {}", msg)
            }
            StorageError::DataNotFound => write!(f, "Data Not Found"),
        }
    }
}
