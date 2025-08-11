use crate::dht::{
    DhtNode,
    peer::PeerInfo,
    rpc::utils::{send_find_rpc, send_store_rpc},
    storage::StoredValue,
};

impl DhtNode {
    pub async fn replicate_to_peers_store(
        &self,
        key: Vec<u8>,
        value: Vec<u8>,
        peers: Vec<PeerInfo>,
    ) -> usize {
        let mut successes = 0;
        for peer in peers {
            if peer.addr == self.addr {
                successes += 1;
                continue;
            }

            if send_store_rpc(self, peer.addr, key.clone(), value.clone())
                .await
                .is_ok()
            {
                successes += 1;
            }
        }
        successes
    }

    pub async fn replicate_to_peers_find(
        &self,
        found_values: &mut Vec<StoredValue>,
        key: Vec<u8>,
        peers: Vec<PeerInfo>,
    ) -> usize {
        let mut successes = 0;
        for peer in peers {
            if send_find_rpc(self, found_values, peer.addr, key.clone())
                .await
                .is_ok()
            {
                successes += 1;
            }
        }
        successes
    }
}
