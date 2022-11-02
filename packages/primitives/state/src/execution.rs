use kp_trie::MemoryDB;

use crate::{TrieBackend, Ext};

/// Trie backend with in-memory storage.
pub type InMemoryBackend<H> = TrieBackend<MemoryDB<H>, H>;

pub(crate) type CallResult<E> = Result<Vec<u8>, E>;

pub struct StateMachine {

}

impl StateMachine {
    pub fn call() {
        
    }

    pub fn create() {
        
    }
}