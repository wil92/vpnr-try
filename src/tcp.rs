use std::net::{TcpListener, TcpStream};
use std::thread;

pub mod protocol;

pub fn tcp_client(
    client_port: i32,
    server_port: i32,
    f: fn(&mut TcpStream, &mut TcpStream),
    f2: fn(&mut TcpStream, &mut TcpStream),
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{client_port}")).unwrap();

    let to_server_stream = TcpStream::connect(format!("127.0.0.1:{server_port}")).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut stream_read = stream.try_clone().expect("Error clonning");
                let mut stream_write = stream.try_clone().expect("Error clonning");
                let mut server_read = to_server_stream.try_clone().expect("Error clonning");
                let mut server_write = to_server_stream.try_clone().expect("Error clonning");
                thread::spawn(move || {
                    f(&mut stream_read, &mut server_write);
                });
                thread::spawn(move || {
                    f2(&mut stream_write, &mut server_read);
                });
            }
            Err(_) => println!("couldn't get client: "),
        }
    }
}

pub fn tcp_server(
    port: i32,
    f: fn(&mut TcpStream),
    f2: fn(&mut TcpStream),
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{port}")).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut stream_read = stream.try_clone().expect("Error clonning");
                let mut stream_write = stream.try_clone().expect("Error clonning");
                thread::spawn(move || {
                    f(&mut stream_read);
                });
                thread::spawn(move || {
                    f2(&mut stream_write);
                });
            }
            Err(_) => println!("couldn't get client: "),
        }
    }
}
