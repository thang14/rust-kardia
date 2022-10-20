use hash_db::Hasher;
use kp_core::hexdisplay::HexDisplay;
use kp_core::traits::{
    CodeExecutor, ReadRuntimeVersionExt, RuntimeCode, SpawnNamed, TaskExecutorExt,
};
use kp_externalities::Extensions;
use kp_trie::MemoryDB;
use tracing::trace;

use crate::{
    ext::Ext, Backend, OverlayedChanges, StateMachineStats, StorageTransactionCache, TrieBackend,
};

/// Trie backend with in-memory storage.
pub type InMemoryBackend<H> = TrieBackend<MemoryDB<H>, H>;

pub(crate) type CallResult<E> = Result<Vec<u8>, E>;

const PROOF_CLOSE_TRANSACTION: &str = "\
		Closing a transaction that was started in this function. Client initiated transactions
		are protected from being closed by the runtime. qed";

/// The KardiaChain state machine.
pub struct StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    B: Backend<H>,
{
    backend: &'a B,
    exec: &'a Exec,
    method: &'a str,
    call_data: &'a [u8],
    overlay: &'a mut OverlayedChanges,
    extensions: Extensions,
    storage_transaction_cache: Option<&'a mut StorageTransactionCache<B::Transaction, H>>,
    runtime_code: &'a RuntimeCode<'a>,
    stats: StateMachineStats,
    /// The hash of the block the state machine will be executed on.
    ///
    /// Used for logging.
    parent_hash: Option<H::Out>,
}

impl<'a, B, H, Exec> Drop for StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    B: Backend<H>,
{
    fn drop(&mut self) {
        self.backend.register_overlay_stats(&self.stats);
    }
}

impl<'a, B, H, Exec> StateMachine<'a, B, H, Exec>
where
    H: Hasher,
    H::Out: Ord + 'static + codec::Codec,
    Exec: CodeExecutor + Clone + 'static,
    B: Backend<H>,
{
    /// Creates new state machine.
    pub fn new(
        backend: &'a B,
        overlay: &'a mut OverlayedChanges,
        exec: &'a Exec,
        method: &'a str,
        call_data: &'a [u8],
        mut extensions: Extensions,
        runtime_code: &'a RuntimeCode,
        spawn_handle: impl SpawnNamed + Send + 'static,
    ) -> Self {
        extensions.register(ReadRuntimeVersionExt::new(exec.clone()));
        extensions.register(TaskExecutorExt::new(spawn_handle));

        Self {
            backend,
            exec,
            method,
            call_data,
            extensions,
            overlay,
            storage_transaction_cache: None,
            runtime_code,
            stats: StateMachineStats::default(),
            parent_hash: None,
        }
    }

    /// Use given `cache` as storage transaction cache.
    ///
    /// The cache will be used to cache storage transactions that can be build while executing a
    /// function in the runtime. For example, when calculating the storage root a transaction is
    /// build that will be cached.
    pub fn with_storage_transaction_cache(
        mut self,
        cache: Option<&'a mut StorageTransactionCache<B::Transaction, H>>,
    ) -> Self {
        self.storage_transaction_cache = cache;
        self
    }

    /// Set the given `parent_hash` as the hash of the parent block.
    ///
    /// This will be used for improved logging.
    pub fn set_parent_hash(mut self, parent_hash: H::Out) -> Self {
        self.parent_hash = Some(parent_hash);
        self
    }

    pub fn execute(&mut self) -> CallResult<Exec::Error> {
        self.overlay.start_transaction();
        let result = self.execute_aux();
        if result.is_err() {
            self.overlay
                .rollback_transaction()
                .expect(PROOF_CLOSE_TRANSACTION);
            result
        } else {
            self.overlay
                .commit_transaction()
                .expect(PROOF_CLOSE_TRANSACTION);
            result
        }
    }

    fn execute_aux(&mut self) -> CallResult<Exec::Error> {
        let mut cache = StorageTransactionCache::default();
        let cache = match self.storage_transaction_cache.as_mut() {
            Some(cache) => cache,
            None => &mut cache,
        };

        self.overlay
            .enter_runtime()
            .expect("StateMachine is never called from the runtime; qed");

        let mut ext = Ext::new(
            self.overlay,
            cache,
            self.backend,
            Some(&mut self.extensions),
        );
        let ext_id = ext.id;

        trace!(
            target: "state",
            ext_id = %HexDisplay::from(&ext_id.to_le_bytes()),
            method = %self.method,
            parent_hash = %self.parent_hash.map(|h| format!("{:?}", h)).unwrap_or_else(|| String::from("None")),
            input = ?HexDisplay::from(&self.call_data),
            "Call",
        );

        let (result, was_native) =
            self.exec
                .call(&mut ext, self.runtime_code, self.method, self.call_data);

        self.overlay
            .exit_runtime()
            .expect("Runtime is not able to call this function in the overlay; qed");

        trace!(
            target: "state",
            ext_id = %HexDisplay::from(&ext_id.to_le_bytes()),
            ?was_native,
            ?result,
            "Return",
        );

        result
    }
}
