/*
   Copyright 2023 Guillermo Gonzalez

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use std::env;

use network::{Client, Server};
use iptables_rule;
use std::io;

pub mod network;
pub mod iptable_rule;

fn main() -> io::Result<()> {
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

    let mut client_ip: String = String::new();
    let mut port: String = String::new();
    let stdin = io::stdin();
    
    print!("input the client ip: ");
    stdin.lock().read_line(&mut client_ip).unwrap();
    print!("input the client port: ");
    stdin.lock().read_line(&mut client_port).unwrap();

    let client_port: i32 = port.trim().parse().expect("Input not an integer");

    iptable_rule::routing_rules(client_ip, client_port);

    Ok(())
}

fn start_server() {
    let server = Server::new();
    server.run(4334);
}

fn start_client() {
    let client = Client::new();
    client.run(4333, 4334);
}
