use std::net::SocketAddr;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{dht::DhtNode, helpers::now};

pub(super) fn serialize_value(value: &StoredValue) -> anyhow::Result<Vec<u8>> {
    bincode::serialize(value).context("Failed to serialize stored value")
}

pub(super) fn deserialize_value(data: &[u8]) -> anyhow::Result<StoredValue> {
    bincode::deserialize(data).context("Failed to deserialize stored value in storage")
}

pub(super) fn create_stored_value(
    data: Vec<u8>,
    addr: SocketAddr,
    is_replica: bool,
    ttl: Option<u64>,
) -> StoredValue {
    StoredValue {
        data,
        version: now(),
        last_node: addr,
        is_replica,
        expiration: ttl.map(|t| now() + t),
        original_nodes: if is_replica { vec![] } else { vec![addr] },
    }
}

pub(super) fn find_in_local_storage(
    node: &DhtNode,
    found_values: &mut Vec<StoredValue>,
    key: Vec<u8>,
) {
    if let Some(value) = node.storage.get(&key) {
        if let Ok(stored) = deserialize_value(&value) {
            let current_time = now();
            if stored.is_valid(current_time) {
                found_values.push(stored);
            } else {
                node.storage.remove(&key);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredValue {
    pub data: Vec<u8>,
    pub version: u64,
    pub last_node: SocketAddr,
    pub is_replica: bool,
    pub expiration: Option<u64>,
    pub original_nodes: Vec<SocketAddr>,
}

impl StoredValue {
    pub fn is_valid(&self, current_time: u64) -> bool {
        self.expiration.map_or(true, |e| e > current_time)
    }
}
