use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

use crate::tcp::protocol;

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
            let mut buf = [0; 200];
            match client.read(&mut buf) {
                Ok(ct) => {
                    if ct == 0 {
                        break;
                    }
                    let mut extra_buf: Vec<u8> = Vec::new();
                    for i in 0..ct {
                        extra_buf.push(buf[i]);
                    }
                    client.write_all(&extra_buf).unwrap();
                }
                Err(_) => {
                    break;
                }
            }
        },
        move |_| {},
    );
}

fn start_client() {
    println!("Client start");

    tcp::tcp_client(
        4333,
        4334,
        move |stream: &mut TcpStream,
              server: &mut TcpStream,
              id_connection: u16,
              streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>| loop {
            let mut buf = [0; 200];
            match stream.read(&mut buf) {
                Ok(ct) => {
                    if ct == 0 {
                        break;
                    }

                    let messages = protocol::code_string(&buf, ct, id_connection);

                    for msg in messages {
                        server.write_all(&msg).unwrap();
                    }
                }
                Err(_) => {
                    let mut streams_shared_ref = streams_shared.lock().unwrap();
                    streams_shared_ref.remove(&id_connection);
                    break;
                }
            }
        },
        move |streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>, server: &mut TcpStream| {
            let mut remain: Vec<u8> = Vec::new();
            loop {
                let mut buf = [0; 200];
                match server.read(&mut buf) {
                    Ok(ct) => {
                        if ct == 0 {
                            break;
                        }

                        let mut ext_buf: Vec<u8> = Vec::new();
                        ext_buf.append(&mut remain);
                        for i in 0..ct {
                            ext_buf.push(buf[i]);
                        }

                        let (messages, rd) = protocol::decode_string(&ext_buf, ext_buf.len());
                        for msg in messages {
                            let streams_shared_ref = streams_shared.lock().unwrap();
                            match streams_shared_ref.get(&msg.1) {
                                Some(mut stream) => {
                                    stream.write_all(&msg.0).unwrap();
                                }
                                None => {}
                            }
                        }

                        if rd != ext_buf.len() {
                            for i in rd..ext_buf.len() {
                                remain.push(ext_buf[i]);
                            }
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
        },
    );
}
