use std::net::SocketAddr;

use tokio::time::timeout;

use crate::dht::{DhtNode, rpc::DhtRpc};

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
