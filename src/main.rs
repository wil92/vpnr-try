use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;

pub mod tcp;

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
    println!("Server start");

    tcp::tcp_server(
        4334,
        move |client: &mut TcpStream| loop {
            let mut buf = [0; 10];
            client.read(&mut buf).unwrap();
            client.write(&buf).unwrap();
        },
        move |_| {},
    );
}

fn start_client() {
    println!("Client start");

    tcp::tcp_client(
        4333,
        4334,
        move |stream: &mut TcpStream, server: &mut TcpStream| loop {
            let mut buf = [0; 10];
            stream.read(&mut buf).unwrap();

            server.write(&buf).unwrap();
            println!("read: {:?}", buf);
        },
        move |stream: &mut TcpStream, server: &mut TcpStream| loop {
            let mut buf = [0; 10];
            server.read(&mut buf).unwrap();

            stream.write(&buf).unwrap();
        },
    );
}
