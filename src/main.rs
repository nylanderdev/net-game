#[cfg(test)]
mod test;
#[macro_use]
extern crate lazy_static;

use crate::game::Client;
use crate::net::{Connection, SmartProtocol};
use std::net::{TcpListener, TcpStream};

mod game;
mod misc;
mod net;

use crate::game::Server;
use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage error");
        exit(1);
    }
    if args[0] == "-host" {
        std::thread::spawn(server_main);
        client_main(&"localhost".to_string())
    } else {
        client_main(&args[0])
    }
}

fn server_main() {
    let listener = TcpListener::bind("0.0.0.0:1337").unwrap();
    let (remote1, _address) = listener.accept().unwrap();
    let (remote2, _address) = listener.accept().unwrap();
    let conn1 = Connection::<SmartProtocol>::from_socket(remote1);
    let conn2 = Connection::<SmartProtocol>::from_socket(remote2);
    let mut server = Server::new(conn1, conn2);
    server.main();
}

fn client_main(address: &String) {
    let remote = TcpStream::connect(format!("{}:1337", address)).unwrap();
    let conn = Connection::<SmartProtocol>::from_socket(remote);
    let client = Client::new();
    client.main(conn);
}
