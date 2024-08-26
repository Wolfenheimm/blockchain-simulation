pub trait Extrinsic {
    fn weight(&self) -> u64;
}

pub struct Transaction {
    pub value: u64,
}

impl Extrinsic for Transaction {
    fn weight(&self) -> u64 {
        self.value
    }
}

//enum for the extrinsic defining the type -> Transfer/Burn
