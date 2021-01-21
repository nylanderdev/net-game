#[cfg(test)]
mod test;

use crate::game::Client;
use crate::net::{Connection, SmartProtocol, DumbProtocol};
use std::net::{TcpListener, TcpStream};

mod game;
mod net;

use crate::game::Server;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        panic!("Usage error");
    }
    if args[0] == "-host" {
        std::thread::spawn(server_main);
        client_main(&"localhost".to_string(), false)
    } else {
        client_main(&args[0], false)
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

fn client_main(address: &String, slow: bool) {
    let remote = TcpStream::connect(format!("{}:1337", address)).unwrap();
    let conn = Connection::<SmartProtocol>::from_socket(remote);
    let client = Client::new();
    client.main(conn, slow);
}
