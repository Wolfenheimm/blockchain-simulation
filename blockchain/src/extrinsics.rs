pub trait ExtrinsicTrait {
    fn weight(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct Extrinsic {
    pub value: u64,
}

impl ExtrinsicTrait for Extrinsic {
    fn weight(&self) -> u64 {
        self.value
    }
}

//enum for the extrinsic defining the type -> Transfer/Burn
