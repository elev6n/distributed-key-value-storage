use std::net::SocketAddr;

use tokio::time::timeout;

use crate::{
    dht::{
        DhtNode,
        rpc::DhtRpc,
        storage::{StoredValue, deserialize_value},
    },
    helpers::now,
};

pub async fn send_store_rpc(
    node: &DhtNode,
    peer: SocketAddr,
    key: Vec<u8>,
    value: Vec<u8>,
) -> anyhow::Result<()> {
    match timeout(
        node.config.operation_timeout,
        node.send_rpc(peer, DhtRpc::Store(key, value)),
    )
    .await
    {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => {
            node.metrics.inc_rpc_failures();
            Err(e)
        }
        Err(_) => {
            node.metrics.inc_rpc_failures();
            Err(anyhow::anyhow!("Store operation timeout"))
        }
    }
}

pub async fn send_find_rpc(
    node: &DhtNode,
    found_values: &mut Vec<StoredValue>,
    addr: SocketAddr,
    key: Vec<u8>,
) -> anyhow::Result<()> {
    match node.send_rpc(addr, DhtRpc::FindValue(key.clone())).await {
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
            node.metrics.inc_rpc_failures();
            Err(e)
        }
    }
}
