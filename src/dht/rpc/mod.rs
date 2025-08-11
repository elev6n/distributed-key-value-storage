pub(super) mod utils;

use serde::{Deserialize, Serialize};

use crate::dht::{node::NodeId, peer::PeerInfo};

/// Remote Procedure Calls (RPCs) used in DHT communication.
///
/// These messages are echanged between nodes to implement the DHT protocol.
#[derive(Debug, Serialize, Deserialize)]
pub enum DhtRpc {
    /// Ping request (check if node is alive)
    Ping,
    /// Ping response
    Pong,
    /// Request to fing nodes closest to a given key
    FindNode(NodeId),
    /// Response containing closest known nodes
    FindNodeResponse(Vec<PeerInfo>),
    /// Request to find a value by key
    FindValue(Vec<u8>),
    /// Response containing found value (if exists)
    FindValueResponse(Option<Vec<u8>>),
    /// Request to store a key-value pair
    Store(Vec<u8>, Vec<u8>),
}
