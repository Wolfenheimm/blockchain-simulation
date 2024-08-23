# blockchain-simulation

**What you should walk away with**

- Strong understanding of high level blockchain components and concepts. This includes but is not limited to, understanding the roles and responsibilities of each component of your blockchain protocol (e.g. state transition function, state, blocks, transactions etc.).
- Rust experience touching many advanced concepts, including but not limited to **traits**, **associated types**, **trait bounds**, **concrete types**, **type aliases**.
- [Defensive programming](https://en.wikipedia.org/wiki/Defensive_programming#:~:text=Defensive%20programming%20is%20an%20approach,approved%20in%20a%20code%20audit.) mindset.
- Clear and concise documentation of the blockchain protocol and components.

**Resources**

- [Rust book](https://doc.rust-lang.org/book/)
- [polkadot-sdk](https://github.com/paritytech/polkadot-sdk/tree/master)
- [Polkadot-sdk docs](https://paritytech.github.io/polkadot-sdk/master/polkadot_sdk_docs/index.html)
- Blockchain diagram from excalidraw

## Requirements

Your main function should be able to initialize state and produce blocks.

### Blockchain

- Blockchain protocol should include the following components (non-exhaustive):
  - State transition function (STF)
  - [Block](https://github.com/paritytech/polkadot-sdk/blob/master/substrate/frame/system/src/lib.rs#L925) and Header 
  - Transactions
  - Agnostic State (i.e. the state should not know what is inside it)
  - Encoding and decoding state
  - Hashes
  - Block and transaction weight (i.e. determining when a block is full based on the amount of total accumulated transactions weight is in the block)
- Your blockchain should be configuratble at startup (e.g. Substrate runtime configuration (e.g. pallet `Config` trait))
- Block import and block execution
- Consensus protocol (i.e. block production and finalization)
- Requirements for STF
  - Should inherit the configuration (e.g. should have a configurable parameter dictating the maximum weight of a block) 
  - Defensive programming validations (e.g. parent block exists)
  - Transfering balance between accounts and tracking balances
  - Tracking total issuance
  - Tracking extrinsics
  - Uncommitted changes to state if anything fails in the STF
  - Agnostic over storage layer

### Rust

- You should have a workspace with a couple or more workspace member projects
  - Workspace members should be libraries used by your main blockchain protocol to accomplish certain things (e.g. storage)
- You should be using all the following rust elements extensively
  - Traits and inheritance
  - Associated types and trait bounds
  - Trait methods with appriopriate use of `&self` and `&mut self`
  - Error handling
  - Modules
  - Structs & Enums
- Tests should cover different outcomes (succeeding or failing block execution)
  - Tests should also be written for your libraries

### Documentation & Diagrams

- Components should be documented (e.g. struct fields documented)
- Libraries should be documented (i.e. using `//!` comment notation)
