use crate::Config;

/// State Transition Function
trait Stf<T: Config> {
    /// Validate a block before executing it.
    fn validate_block(&self, block: T::Block) -> Result<(), Error>;
    /// Execute a block and update state.
    fn execute_block(&self, block: T::Block);
}
