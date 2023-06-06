use std::collections::HashMap;
use std::env;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;

use nix::sys::socket::{self, sockopt};
use std::os::fd::AsRawFd;

use crate::tcp::protocol;

pub mod tcp;

const CONNECTION_FAIL: u8 = 1;

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
        move |client: &mut TcpStream| {
            let redirection_map: HashMap<u16, TcpStream> = HashMap::new();
            let redirection_map_ref = Arc::new(Mutex::new(redirection_map));

            let mut remain: Vec<u8> = Vec::new();
            loop {
                let mut buf = [0; 200];
                match client.read(&mut buf) {
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
                            let msg_id = msg.1;
                            // let flags = msg.2;
                            let addr = msg.3;
                            let port = msg.4;

                            println!(
                                "{}, addr: {}.{}.{}.{}, port: {}",
                                addr,
                                addr & 255,
                                (addr >> 8) & 255,
                                (addr >> 16) & 255,
                                (addr >> 24) & 255,
                                port
                            );

                            let mut stream_dest: TcpStream;
                            let stream_exist: bool;
                            {
                                stream_exist =
                                    redirection_map_ref.lock().unwrap().contains_key(&msg_id);
                            }
                            if !stream_exist {
                                match TcpStream::connect(format!(
                                    "{}.{}.{}.{}:{}",
                                    addr & 255,
                                    (addr >> 8) & 255,
                                    (addr >> 16) & 255,
                                    (addr >> 24) & 255,
                                    port
                                )) {
                                    Ok(stream) => {
                                        {
                                            let mut rf = redirection_map_ref.lock().unwrap();
                                            rf.insert(msg_id, stream);
                                            stream_dest = rf
                                                .get(&msg_id)
                                                .unwrap()
                                                .try_clone()
                                                .expect("error");
                                        }

                                        let redirection_map_ref_clone = redirection_map_ref.clone();
                                        let msg_id_ref = msg_id;
                                        let mut stream_ref = stream_dest
                                            .try_clone()
                                            .expect("Can't clone redirection stream");
                                        let mut client_ref = client.try_clone().expect("asdf");
                                        thread::spawn(move || loop {
                                            let mut buf = [0; 200];
                                            match stream_ref.read(&mut buf) {
                                                Ok(ct) => {
                                                    if ct == 0 {
                                                        let mut rf = redirection_map_ref_clone
                                                            .lock()
                                                            .unwrap();
                                                        rf.remove(&msg_id_ref);

                                                        let close_buff = protocol::code_block(
                                                            b"",
                                                            0,
                                                            msg_id_ref,
                                                            CONNECTION_FAIL,
                                                            0,
                                                            0,
                                                        );
                                                        client_ref.write_all(&close_buff).unwrap();
                                                        break;
                                                    }

                                                    let messages = protocol::code_string(
                                                        &buf, ct, msg_id_ref, 0, 0, 0,
                                                    );

                                                    for msg in messages {
                                                        if let Err(_) = client_ref.write_all(&msg) {
                                                            break;
                                                        }
                                                    }
                                                }
                                                Err(_) => {
                                                    println!("asdf");
                                                }
                                            }
                                        });
                                    }
                                    Err(_) => {
                                        // send disconnection to the client
                                        println!("error");
                                        continue;
                                    }
                                }
                            } else {
                                let rf = redirection_map_ref.lock().unwrap();
                                stream_dest = rf.get(&msg_id).unwrap().try_clone().expect("error");
                            }

                            stream_dest.write_all(&msg.0).unwrap();
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

                    let sock_fd = stream.as_raw_fd();
                    let original_addr = socket::getsockopt(sock_fd, sockopt::OriginalDst).unwrap();
                    println!(
                        "{}, addr: {}.{}.{}.{}, port: {}",
                        original_addr.sin_addr.s_addr,
                        original_addr.sin_addr.s_addr & 255,
                        (original_addr.sin_addr.s_addr >> 8) & 255,
                        (original_addr.sin_addr.s_addr >> 16) & 255,
                        (original_addr.sin_addr.s_addr >> 24) & 255,
                        u16::from_be(original_addr.sin_port)
                    );

                    let messages = protocol::code_string(
                        &buf,
                        ct,
                        id_connection,
                        0,
                        original_addr.sin_addr.s_addr,
                        u16::from_be(original_addr.sin_port),
                    );

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
                            let mut streams_shared_ref = streams_shared.lock().unwrap();
                            let flags = msg.2;

                            match streams_shared_ref.get(&msg.1) {
                                Some(mut stream) => {
                                    if flags & CONNECTION_FAIL != 0 {
                                        if let Ok(_) = stream.shutdown(std::net::Shutdown::Both) {}
                                        streams_shared_ref.remove(&msg.1).unwrap();
                                        continue;
                                    }
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
