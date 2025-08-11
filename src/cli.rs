use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[command(version, about)]
pub struct Cli {
    /// Address to bind this node to (e.g. 127.0.0.1:8080)
    #[arg(long, short)]
    pub addr: SocketAddr,

    /// Known peers to bootstrap the network (comma separated)
    #[arg(long, short)]
    pub peers: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Store a key-value pair in the DHT
    Store { key: String, value: String },

    /// Retrieve a value from the DHT
    Get { key: String },

    /// List all known peers in the routing table
    Peers,

    /// Show DHT statistics
    Stats,
}
