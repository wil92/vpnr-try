use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::thread;

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
        move |stream: &mut TcpStream, client: &mut TcpStream| {
            let mut buf = [0; 10];
            stream.read(&mut buf).unwrap();

            client.write(&buf).unwrap();
            println!("read: {:?}", buf);
        },
        |_, _| {},
    );
}

fn start_client() {
    println!("Client start");

    let mut to_server_stream = TcpStream::connect("127.0.0.1:4334").unwrap();

    tcp::tcp_server(
        4333,
        &mut to_server_stream,
        move |stream: &mut TcpStream, server: &mut TcpStream| {
            let mut buf = [0; 10];
            stream.read(&mut buf).unwrap();

            server.write(&buf).unwrap();
            println!("read: {:?}", buf);
        },
        move |stream: &mut TcpStream, server: &mut TcpStream| {
            let mut buf = [0; 10];
            server.read(&mut buf).unwrap();

            stream.write(&buf);
        },
    );
}
