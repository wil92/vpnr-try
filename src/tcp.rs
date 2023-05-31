use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod protocol;

pub fn tcp_server(port: i32, server: &mut TcpStream, f: fn(&mut TcpStream, &mut TcpStream), f2: fn(&mut TcpStream, &mut TcpStream)) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                thread::spawn(move || {
                    f(&mut stream, server);
                });
                thread::spawn(move || {
                    f2(&mut stream, server);
                });
            }
            Err(_) => println!("couldn't get client: "),
        }
    }
}
