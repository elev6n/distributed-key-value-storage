use crate::{dht::{peer::PeerInfo, rpc::DhtRpc, storage::{deserialize_value, StoredValue}, DhtNode}, helpers::now};

impl DhtNode {
    pub async fn query_peers_for_value(
        &self,
        key: Vec<u8>,
        peers: Vec<PeerInfo>,
    ) -> Vec<StoredValue> {
        let mut found_values = Vec::new();
        for peer in peers {
            if let Ok(DhtRpc::FindValueResponse(Some(data))) =  
                self.send_rpc(peer.addr, DhtRpc::FindValue(key.clone())).await
            {
                if let Ok(stored) = deserialize_value(&data) {
                    let current_time = now();
                    if stored.is_valid(current_time) {
                        found_values.push(stored);
                    }
                }
            }
        }
        found_values
    }
}