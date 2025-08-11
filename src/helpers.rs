use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::dht::{
    config::{DhtConfig, ReplicationConfig, StorageConfig}, DhtNode
};

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn create_test_node(port: u16) -> DhtNode {
    let config = DhtConfig {
        replication: ReplicationConfig {
            factor: 5,
            check_interval: Duration::from_secs(60),
            parallelism: 3,
        },
        storage: StorageConfig {
            max_entries: 2048,
            default_ttl: 1,
            expiration_check_interval: 1,
        },
        ..Default::default()
    };

    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    DhtNode::new(addr, Some(config))
}
