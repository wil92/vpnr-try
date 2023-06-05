use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

pub mod protocol;

pub fn tcp_client(
    client_port: i32,
    server_port: i32,
    f: fn(
        &mut TcpStream,
        &mut TcpStream,
        id_connection: u16,
        streams_shared: Arc<Mutex<HashMap<u16, TcpStream>>>,
    ),
    f2: fn(Arc<Mutex<HashMap<u16, TcpStream>>>, &mut TcpStream),
) {
    let listener = TcpListener::bind(format!("127.0.0.1:{client_port}")).unwrap();

    let to_server_stream = TcpStream::connect(format!("127.0.0.1:{server_port}")).unwrap();

    let mut id_cont: u16 = 0;
    let apps_map: HashMap<u16, TcpStream> = HashMap::new();
    let shared_map = Arc::new(Mutex::new(apps_map));

    let mut server_read = to_server_stream.try_clone().expect("Error clonning");
    let shared_map_copy = shared_map.clone();
    thread::spawn(move || {
        f2(shared_map_copy, &mut server_read);
    });

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let mut shared_map_ref = shared_map.lock().unwrap();

                shared_map_ref.insert(id_cont, stream);

                let mut stream_read = shared_map_ref
                    .get(&id_cont)
                    .unwrap()
                    .try_clone()
                    .expect("Error clonning application stream");
                let mut server_write = to_server_stream.try_clone().expect("Error clonning");

                let shared_map_copy = shared_map.clone();
                thread::spawn(move || {
                    f(
                        &mut stream_read,
                        &mut server_write,
                        id_cont,
                        shared_map_copy,
                    );
                });

                id_cont += 1;
            }
            Err(_) => println!("couldn't get client: "),
        }
    }
}

pub fn tcp_server(port: i32, f: fn(&mut TcpStream), f2: fn(&mut TcpStream)) {
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
