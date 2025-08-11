use std::net::SocketAddr;

use crate::{
    dht::{
        DhtNode,
        peer::PeerInfo,
        rpc::DhtRpc,
        storage::{StoredValue, deserialize_value},
    },
    helpers::now,
};

impl DhtNode {
    pub async fn query_peers_for_value(
        &self,
        found_values: &mut Vec<StoredValue>,
        key: Vec<u8>,
        peers: Vec<PeerInfo>,
    ) -> usize {
        let mut successes = 0;

        for peer in peers {
            if self
                .send_query_peers(found_values, key.clone(), peer.addr)
                .await
                .is_ok()
            {
                successes += 1;
            }
        }

        successes
    }

    async fn send_query_peers(
        &self,
        found_values: &mut Vec<StoredValue>,
        key: Vec<u8>,
        addr: SocketAddr,
    ) -> anyhow::Result<()> {
        match self.send_rpc(addr, DhtRpc::FindValue(key.clone())).await {
            Ok(DhtRpc::FindValueResponse(Some(data))) => {
                if let Ok(stored) = deserialize_value(&data) {
                    let current_time = now();
                    if stored.is_valid(current_time) {
                        found_values.push(stored);
                    }
                }
                Ok(())
            }
            Ok(_) => Ok(()),
            Err(e) => {
                self.metrics.inc_rpc_failures();
                Err(e)
            }
        }
    }
}
