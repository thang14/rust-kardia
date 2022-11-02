extern crate alloc;

pub mod transaction;
pub mod  log;
pub mod  account;

// Alias for `Vec<u8>`. This type alias is necessary for rlp-derive to work correctly.
type Bytes = alloc::vec::Vec<u8>;