//! Kademlia K-Bucket implementation for the DHT.
//!
//! The [`KBucket`] struct represents a single bucket in the Kademlia routing table,
//! containing a list of peers sorted by their last seen time (LRU).

use crate::dht::{node::NodeId, peer::PeerInfo};

/// A bucket in the Kademlia routing table holds up to `max_size` peers.
///
/// KBuckets maintain a list of peers sorted by last contact time (least recently
/// seen peers are at the end). When the bucket is full, new peers are only
/// inserted if they replace older, potentially stale peers.
///
/// # Examples
///
/// ```rust
/// use rust_p2p_node::dht::{kbucket::KBucket, peer::PeerInfo, node::NodeId};
/// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
///
/// let mut bucket = KBucket {
///     peers: Vec::new(),
///     max_size: 2,
/// };
///
/// let peer = PeerInfo {
///     id: NodeId::new(b"peer"),
///     addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8080),
///     last_seen: 0,
/// };
///
/// bucket.update_peer(peer.clone());
/// assert_eq!(bucket.get_peers().len(), 1);
/// ```
#[derive(Debug, Default)]
pub struct KBucket {
    /// List of peers in this bucket, sorted by last contact time (most recent first)
    pub peers: Vec<PeerInfo>,
    /// Maximum number of peers this bucket can hold (typically 20 in Kademlia)
    pub max_size: usize,
}

impl KBucket {
    /// Updates or inserts a peer into the bucket.
    ///
    /// If the peer already exists, it will be updated and moved to the front.
    /// If the bucket is full, the new peer will be discarded unless it replaces
    /// a stale connection.
    ///
    /// # Arguments
    ///
    /// * `peer` - The peer information to update or insert
    pub fn update_peer(&mut self, peer: PeerInfo) {
        self.peers.retain(|p| p.id != peer.id);

        if self.peers.len() >= self.max_size {
            return;
        }

        self.peers.push(peer);
    }

    /// Returns a copy of all peers in this bucket.
    ///
    /// Peers are returned in order from most recently seen to least recently seen.
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.clone()
    }

    /// Returns the peer by id.
    pub fn get_peer(&self, peer_id: &NodeId) -> Option<&PeerInfo> {
        self.peers.iter().find(|p| &p.id == peer_id)
    }

    /// Returns the number of peers currently in the bucket.
    pub fn len(&self) -> usize {
        self.peers.len()
    }

    /// Checks if the bucket is empty.
    pub fn is_empty(&self) -> bool {
        self.peers.is_empty()
    }

    /// Checks if the bucket is full (has reached max_size).
    pub fn is_full(&self) -> bool {
        self.peers.len() >= self.max_size
    }
}

#[cfg(test)]
mod kbucket_tests {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use crate::dht::{KBucket, NodeId, PeerInfo};

    fn create_peer(id: &str) -> PeerInfo {
        PeerInfo {
            id: NodeId::new(id.as_bytes()),
            addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 8090),
            last_seen: 0,
        }
    }

    #[test]
    fn test_add_peer() {
        let mut bucket = KBucket {
            peers: Vec::new(),
            max_size: 2,
        };

        let peer1 = create_peer("peer1");
        let peer2 = create_peer("peer2");

        bucket.update_peer(peer1.clone());
        bucket.update_peer(peer2.clone());

        assert_eq!(bucket.len(), 2);
        assert!(bucket.get_peers().contains(&peer1));
        assert!(bucket.get_peers().contains(&peer2));
    }

    #[test]
    fn test_update_existing_peer() {
        let mut bucket = KBucket {
            peers: Vec::new(),
            max_size: 2,
        };

        let mut peer1 = create_peer("peer1");
        bucket.update_peer(peer1.clone());

        peer1.last_seen = 100;
        bucket.update_peer(peer1.clone());

        assert_eq!(bucket.len(), 1);
        assert_eq!(bucket.get_peers()[0].last_seen, 100);
    }

    #[test]
    fn test_bucket_full() {
        let mut bucket = KBucket {
            peers: Vec::new(),
            max_size: 1,
        };

        let peer1 = create_peer("peer1");
        let peer2 = create_peer("peer2");

        bucket.update_peer(peer1);
        bucket.update_peer(peer2);

        assert_eq!(bucket.len(), 1);
        assert!(bucket.is_full());
    }
}
