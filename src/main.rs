mod app;
mod cli;

use clap::Parser;
use rust_p2p_node::dht::{DhtNode, config::DhtConfig};
use tokio::sync::mpsc;

use crate::{
    app::{AppCommand, DhtApp},
    cli::{Cli, Commands},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let node = DhtNode::new(cli.addr, Some(DhtConfig::default()));
    node.start_maintenance_service().await;

    let (command_sender, command_receiver) = mpsc::channel(32);

    let app_handle = tokio::spawn(async move {
        let app = DhtApp::new(node, command_receiver);
        app.run().await;
    });

    if let Some(command) = cli.command {
        match command {
            Commands::Store { key, value } => {
                command_sender.send(AppCommand::Store(key, value)).await?;
            }
            Commands::Get { key } => {
                command_sender.send(AppCommand::Get(key)).await?;
            }
            Commands::Peers => {
                command_sender.send(AppCommand::ListPeers).await?;
            }
            Commands::Stats => {
                command_sender.send(AppCommand::GetStats).await?;
            }
        }
    } else {
        // Interactive mode
        println!("Running in interactive mode. Type 'help' for commands.");
        let mut input = String::new();
        loop {
            print!("> ");
            std::io::stdin().read_line(&mut input)?;

            let parts: Vec<&str> = input.trim().split_whitespace().collect();
            match parts.as_slice() {
                ["store", key, value] => {
                    command_sender
                        .send(AppCommand::Store(key.to_string(), value.to_string()))
                        .await?;
                }
                ["get", key] => {
                    command_sender
                        .send(AppCommand::Get(key.to_string()))
                        .await?;
                }
                ["peers"] => {
                    command_sender.send(AppCommand::ListPeers).await?;
                }
                ["stats"] => {
                    command_sender.send(AppCommand::GetStats).await?;
                }
                ["help"] => {
                    print_help();
                }
                ["exit"] => break,
                _ => println!("Unknown command. Type 'help' for available commands."),
            }
            input.clear();
        }
    }

    app_handle.abort();

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  store <key> <value> - Store a key-value pair");
    println!("  get <key>           - Retrieve a value by key");
    println!("  peers               - List known peers");
    println!("  stats               - Show DHT statistics");
    println!("  exit                - Exit the application");
}
