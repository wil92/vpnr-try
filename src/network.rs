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

use nix::sys::socket::{self, sockopt, Shutdown};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::network::tcp::TcpClient;

use self::tcp::TcpServer;

pub mod protocol;
pub mod tcp;

// Flags
pub const CONNECTION_FAIL: u8 = 1;

pub struct Client {}

impl Client {
    pub fn new() -> Client {
        Client {}
    }

    pub fn run(self, client_port: i32, server_port: i32) {
        println!("Client start on port: {}", client_port);

        let mut tcp = TcpClient::new(
            client_port,
            server_port,
            move |stream: &mut TcpStream,
                  server: &mut TcpStream,
                  id_connection: u16,
                  streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>| {
                Client::new_app_connection(stream, server, id_connection, streams_shared)
            },
            move |streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
                  server: &mut TcpStream,
                  fd: i32| { Client::server_new_message(streams_shared, server, fd) },
        );

        tcp.connect();
    }

    fn new_app_connection(
        stream: &mut TcpStream,
        server: &mut TcpStream,
        id_connection: u16,
        streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
    ) {
        loop {
            let mut buf = [0; 200];

            match stream.read(&mut buf) {
                Ok(ct) => {
                    if ct == 0 {
                        break;
                    }
                    if let Ok((original_addr, original_port)) = Client::get_original_addr(&stream) {
                        Server::print_new_connection_info(original_addr, original_port);

                        let messages = protocol::code_string(
                            &buf,
                            ct,
                            id_connection,
                            0,
                            original_addr,
                            original_port,
                        );

                        for msg in messages {
                            server.write_all(&msg).unwrap();
                        }
                    } else {
                        Client::remove_app_connection_ref(&streams_shared, id_connection);
                    }
                }
                Err(_) => {
                    Client::remove_app_connection_ref(&streams_shared, id_connection);
                    break;
                }
            }
        }
    }

    fn remove_app_connection_ref(
        streams_shared: &Arc<Mutex<HashMap<u16, TcpStream>>>,
        id_connection: u16,
    ) {
        let mut streams_shared_ref = streams_shared.lock().unwrap();
        streams_shared_ref.remove(&id_connection);
    }

    fn server_new_message(
        streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
        server: &mut TcpStream,
        fd: i32,
    ) {
        let mut remain: Vec<u8> = Vec::new();
        loop {
            let mut buf = [0; 200];
            match server.read(&mut buf) {
                Ok(ct) => {
                    if ct == 0 {
                        socket::shutdown(fd, Shutdown::Both).unwrap();
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
                                if let Ok(_) = stream.write_all(&msg.0) {}
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
    }

    fn get_original_addr(stream: &TcpStream) -> Result<(u32, u16), &'static str> {
        let sock_fd = stream.as_raw_fd();
        if let Ok(original_addr) = socket::getsockopt(sock_fd, sockopt::OriginalDst) {
            println!(
                "{}, addr: {}.{}.{}.{}, port: {}",
                original_addr.sin_addr.s_addr,
                original_addr.sin_addr.s_addr & 255,
                (original_addr.sin_addr.s_addr >> 8) & 255,
                (original_addr.sin_addr.s_addr >> 16) & 255,
                (original_addr.sin_addr.s_addr >> 24) & 255,
                u16::from_be(original_addr.sin_port)
            );

            return Ok((
                original_addr.sin_addr.s_addr,
                u16::from_be(original_addr.sin_port),
            ));
        }
        Err("Fail to extract ORIGINAL_DST")
    }
}

pub struct Server {}

impl Server {
    pub fn new() -> Server {
        Server {}
    }

    pub fn run(&self, port: i32) {
        println!("Server start on port: {}", port);

        let tcp = TcpServer::new(
            port,
            move |client: &mut TcpStream, redirection_map: Arc<Mutex<HashMap<u16, TcpStream>>>| {
                Server::new_client_connection(client, redirection_map);
            },
        );

        tcp.tcp_server();
    }

    fn new_client_connection(
        client: &mut TcpStream,
        redirection_map: Arc<Mutex<HashMap<u16, TcpStream>>>,
    ) {
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

                        let mut stream_dest: TcpStream;
                        let stream_exist: bool;
                        {
                            let redirection_map_clone = redirection_map.clone();
                            stream_exist =
                                redirection_map_clone.lock().unwrap().contains_key(&msg_id);
                        }
                        if !stream_exist {
                            if let Ok(stream_res) = Server::open_new_redirection_connection(
                                msg_id,
                                addr,
                                port,
                                &client,
                                &redirection_map,
                            ) {
                                stream_dest = stream_res;
                            } else {
                                let mut client_ref = client.try_clone().expect("asdf");
                                Server::send_connection_fails_to_client(msg_id, &mut client_ref);
                                continue;
                            }
                        } else {
                            let rf = redirection_map.lock().unwrap();
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
    }

    fn open_new_redirection_connection(
        msg_id: u16,
        addr: u32,
        port: u16,
        client: &TcpStream,
        redirection_map: &Arc<Mutex<HashMap<u16, TcpStream>>>,
    ) -> Result<TcpStream, &'static str> {
        let stream_dest: TcpStream;
        Server::print_new_connection_info(addr, port);
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
                    let mut rf = redirection_map.lock().unwrap();
                    rf.insert(msg_id, stream);
                    stream_dest = rf.get(&msg_id).unwrap().try_clone().expect("error");
                }

                Server::print_new_connection_info(addr, port);

                let redirection_map_ref_clone = redirection_map.clone();
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
                                let mut rf = redirection_map_ref_clone.lock().unwrap();
                                rf.remove(&msg_id_ref);

                                Server::send_connection_fails_to_client(
                                    msg_id_ref,
                                    &mut client_ref,
                                );
                                break;
                            }

                            let messages = protocol::code_string(&buf, ct, msg_id_ref, 0, 0, 0);

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
            Err(_) => return Err("Error connecting to destination"),
        }
        Ok(stream_dest)
    }

    fn send_connection_fails_to_client(msg_id_ref: u16, client_ref: &mut TcpStream) {
        let close_buff = protocol::code_block(b"", 0, msg_id_ref, CONNECTION_FAIL, 0, 0);
        if let Ok(_) = client_ref.write_all(&close_buff) {}
    }

    fn print_new_connection_info(addr: u32, port: u16) {
        println!(
            "{}, addr: {}.{}.{}.{}, port: {}",
            addr,
            addr & 255,
            (addr >> 8) & 255,
            (addr >> 16) & 255,
            (addr >> 24) & 255,
            port
        );
    }
}
