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

use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::os::fd::AsRawFd;
use std::sync::{Arc, Mutex};
use std::thread;

pub struct TcpClient {
    client_port: i32,
    server_port: i32,
    id_cont: u16,
    shared_map: Arc<Mutex<HashMap<u16, TcpStream>>>,
    new_app_connection_event: fn(
        &mut TcpStream,
        &mut TcpStream,
        id_connection: u16,
        streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
    ),
    server_new_message_event: fn(Arc<Mutex<HashMap<u16, TcpStream>>>, &mut TcpStream, fd: i32),
}
impl TcpClient {
    pub fn new(
        client_port: i32,
        server_port: i32,
        new_app_connection_event: fn(
            &mut TcpStream,
            &mut TcpStream,
            id_connection: u16,
            streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
        ),
        server_new_message_event: fn(Arc<Mutex<HashMap<u16, TcpStream>>>, &mut TcpStream, fd: i32),
    ) -> TcpClient {
        TcpClient {
            client_port,
            server_port,
            id_cont: 0,
            shared_map: Arc::new(Mutex::new(HashMap::new())),
            new_app_connection_event,
            server_new_message_event,
        }
    }

    pub fn connect(&mut self) {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.client_port)).unwrap();

        let to_server_stream =
            TcpStream::connect(format!("127.0.0.1:{}", self.server_port)).unwrap();

        let mut server_read = to_server_stream.try_clone().expect("Error clonning");
        let shared_map_copy = self.shared_map.clone();
        let server_new_message_event = self.server_new_message_event;
        let fd = listener.as_raw_fd();
        thread::spawn(move || {
            (server_new_message_event)(shared_map_copy, &mut server_read, fd);
        });

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    self.increase_id_cont();

                    let mut stream_read;
                    {
                        let mut shared_map_ref = self.shared_map.lock().unwrap();

                        shared_map_ref.insert(self.id_cont, stream);

                        stream_read = shared_map_ref
                            .get(&self.id_cont)
                            .unwrap()
                            .try_clone()
                            .expect("Error clonning application stream");
                    }
                    let mut server_write = to_server_stream.try_clone().expect("Error clonning");

                    let shared_map_copy = self.shared_map.clone();
                    let new_app_connection_event = self.new_app_connection_event.clone();
                    let id_cont = self.id_cont;
                    thread::spawn(move || {
                        (new_app_connection_event)(
                            &mut stream_read,
                            &mut server_write,
                            id_cont,
                            shared_map_copy,
                        );
                    });
                }
                Err(_) => {
                    println!("Traffic listener is close.");
                    break;
                }
            }
        }
    }

    fn increase_id_cont(&mut self) {
        self.id_cont += 1;
    }
}

pub struct TcpServer {
    port: i32,
    redirection_map: Arc<Mutex<HashMap<u16, TcpStream>>>,
    new_client_connection_event: fn(&mut TcpStream, Arc<Mutex<HashMap<u16, TcpStream>>>),
}

impl TcpServer {
    pub fn new(
        port: i32,
        new_client_connection_event: fn(&mut TcpStream, Arc<Mutex<HashMap<u16, TcpStream>>>),
    ) -> TcpServer {
        TcpServer {
            port,
            redirection_map: Arc::new(Mutex::new(HashMap::new())),
            new_client_connection_event,
        }
    }

    pub fn tcp_server(&self) {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", self.port)).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let new_client_connection_event = self.new_client_connection_event;
                    let redirection_map = self.redirection_map.clone();
                    let mut stream_ref = stream.try_clone().unwrap();
                    thread::spawn(move || {
                        (new_client_connection_event)(&mut stream_ref, redirection_map);
                    });
                }
                Err(_) => println!("couldn't get client: "),
            }
        }
    }
}
