use std::net::SocketAddr;

use tokio::sync::mpsc;

use rust_p2p_node::dht::DhtNode;

pub struct DhtApp {
    pub node: DhtNode,
    command_receiver: mpsc::Receiver<AppCommand>,
}

pub enum AppCommand {
    Store(String, String),
    Get(String),
    ListPeers,
    GetStats,
}

impl DhtApp {
    pub fn new(node: DhtNode, command_receiver: mpsc::Receiver<AppCommand>) -> Self {
        Self {
            node,
            command_receiver,
        }
    }

    pub async fn run(mut self) {
        println!("DHT node running at {}", self.node.addr);

        if let Some(peers) = self.get_initial_peers().await {
            if let Err(e) = self.node.bootstrap(peers).await {
                eprintln!("Bootstrap failed: {}", e);
            }
        }

        while let Some(cmd) = self.command_receiver.recv().await {
            match cmd {
                AppCommand::Store(key, value) => {
                    self.handle_store(key, value).await;
                }
                AppCommand::Get(key) => {
                    self.handle_get(key).await;
                }
                AppCommand::ListPeers => {
                    self.handle_list_peers().await;
                }
                AppCommand::GetStats => {
                    self.handle_get_stats().await;
                }
            }
        }
    }

    async fn get_initial_peers(&self) -> Option<Vec<SocketAddr>> {
        // In a real app, you might load these from config or CLI
        None
    }

    async fn handle_store(&self, key: String, value: String) {
        match self.node.store(key.into_bytes(), value.into_bytes()).await {
            Ok(_) => println!("Value stored successfully"),
            Err(e) => eprintln!("Failed to store value: {}", e),
        }
    }

    async fn handle_get(&self, key: String) {
        match self.node.find_value(key.into_bytes()).await {
            Some(value) => {
                if let Ok(str_value) = String::from_utf8(value.clone()) {
                    println!("Value: {}", str_value);
                } else {
                    println!("Value (binary): {:?}", value);
                }
            }
            None => println!("Value not found"),
        }
    }

    async fn handle_list_peers(&self) {
        let mut peers = Vec::new();

        for bucket in self.node.routing_table.iter() {
            peers.extend(bucket.value().peers.iter().cloned());
        }

        if peers.is_empty() {
            println!("No known peers");
            return;
        }

        println!("Known peers ({}):", peers.len());
        for peer in peers {
            println!("- ID: {}, Addr: {}", peer.id, peer.addr);
        }
    }

    async fn handle_get_stats(&self) {
        let stats = self.node.get_stats();
        println!("DHT Statistics:");
        println!("- Store operations: {}", stats.store_ops);
        println!("- Successful stores: {}", stats.store_success);
        println!("- Find operations: {}", stats.find_value_ops);
        println!("- Successful finds: {}", stats.find_value_success);
        println!("- RPC requests: {}", stats.rpc_requests);
        println!("- RPC failures: {}", stats.rpc_failures);
        println!("- Known peers: {}", stats.known_peers);
        println!("- Storage size: {}", stats.storage_size);
    }
}
