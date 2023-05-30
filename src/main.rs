use std::env;
use std::net::TcpListener;
use std::io::Read;
use std::thread;

pub mod protocol;

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
    let listener = TcpListener::bind("127.0.0.1:4333").unwrap();

    let mut handlers = Vec::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let handler = thread::spawn(move || {
                    let mut buf = [0; 10];
                    stream.read(&mut buf).unwrap();

                    println!("read: {:?}", buf);
                });

                handlers.push(handler);
            }
            Err(_) => println!("couldn't get client: "),
        }
    }
}

fn start_client() {
    println!("Client start");
}
