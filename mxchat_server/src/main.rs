use std::{net::IpAddr, str::FromStr};

use server_handler::ServerCommandHandler;
use server::{run_server, ServerConfig};

mod server;
mod command_handler;
mod server_handler;
mod user;

fn main() {

    let config = ServerConfig {
        address: IpAddr::from_str("127.0.0.1").unwrap(),
        port: 8080
    };

    let cmd_handler = ServerCommandHandler::new();

    println!("Running server...");
    run_server(cmd_handler, config).unwrap();
}
