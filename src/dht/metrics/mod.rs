pub(super) mod utils;

use std::sync::{atomic::{AtomicU64, Ordering}, Arc};

/// Metrics collection for DHT operations
#[derive(Debug, Default)]
pub struct DhtMetrics {
    /// Number of store operations
    pub store_ops: AtomicU64,
    /// Number of successful store operations
    pub store_success: AtomicU64,
    /// Number of find_value operations
    pub find_value_ops: AtomicU64,
    /// Number of successful find_value operations
    pub find_value_success: AtomicU64,
    /// Number of RPC requests sent
    pub rpc_requests: AtomicU64,
    /// Number of failed RPC requests
    pub rpc_failures: AtomicU64,
    /// Number of peers in routing table
    pub known_peers: AtomicU64,
}

impl DhtMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub fn inc_store_ops(&self) {
        self.store_ops.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_store_success(&self) {
        self.store_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_find_value_ops(&self) {
        self.find_value_ops.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_find_value_success(&self) {
        self.find_value_success.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_rpc_requests(&self) {
        self.rpc_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_rpc_failures(&self) {
        self.rpc_failures.fetch_add(1, Ordering::Relaxed);
    }

    pub fn set_known_peers(&self, count: u64) {
        self.known_peers.store(count, Ordering::Relaxed);
    }
}

/// Snapshot of DHT metrics
#[derive(Debug, Clone)]
pub struct DhtStats {
    pub store_ops: u64,
    pub store_success: u64,
    pub find_value_ops: u64,
    pub find_value_success: u64,
    pub rpc_requests: u64,
    pub rpc_failures: u64,
    pub known_peers: u64,
    pub storage_size: u64,
}