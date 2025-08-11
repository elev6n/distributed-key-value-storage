use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::{dht::node::NodeId, helpers::now};

/// Information about a peer in the DHT network.
///
/// Contains the peer's indentifier, network address, and last contact time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeerInfo {
    /// The node's unique identifier (typically a hash of its address/public key)
    pub id: NodeId,
    /// Network address where the peer can be reached
    pub addr: SocketAddr,
    /// Unix timestamp of last successful communication
    pub last_seen: u64,
}

impl PeerInfo {
    pub fn new(id: NodeId, addr: SocketAddr) -> Self {
        Self {
            id,
            addr,
            last_seen: now(),
        }
    }
}
