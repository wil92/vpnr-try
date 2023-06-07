use std::env;

use network::{Client, Server};

pub mod network;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut run_server = false;
    for it in args {
        if it == "-s" {
            run_server = true;
        }
    }

    if run_server {
        start_server();
    } else {
        start_client();
    }
}

fn start_server() {
    let server = Server::new();
    server.run(4334);
}

fn start_client() {
    let client = Client::new();
    client.run(4333, 4334);
}
