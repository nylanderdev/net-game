#![allow(deprecated)]

use crate::misc::constants::ALL_KEYS;
use crate::net::{Connection, DumbProtocol, Event, Protocol, SmartProtocol};
use ggez::event::KeyCode;
use std::net::{TcpListener, TcpStream};
use std::thread;

fn test_protocol_encode_decode<PROTOCOL: Protocol>(expected: Event) {
    let actual_bytes = PROTOCOL::encode(&expected);
    let actual_option = PROTOCOL::decode(actual_bytes.as_slice());
    assert!(actual_option.is_some());
    assert_eq!(expected, actual_option.unwrap());
}

#[test]
fn dumb_protocol_encode_decode_movement() {
    let expected = Event::Movement(0xff, 1432.0, -1432.0, -13452.0);
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_ready() {
    let expected = Event::Ready;
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_start() {
    let expected = Event::Start;
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_yield() {
    let expected = Event::Yield(u64::max_value());
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_request_movement() {
    let expected = Event::RequestMovement(0xff, 1432.0, -1432.0, -13452.0);
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_custom() {
    let expected = Event::Custom(2131231, vec![234, 3, b'\n', 34, b'\x1b', 32, b'\n']);
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_encode_decode_spawn() {
    let expected = Event::Spawn(u64::max_value());
    test_protocol_encode_decode::<DumbProtocol>(expected);
}

#[test]
fn dumb_protocol_single_event_transfer() {
    fn server_main() {
        let listener = TcpListener::bind("localhost:1337").unwrap();
        let (remote, _address) = listener.accept().unwrap();
        let mut conn = Connection::<DumbProtocol>::from_socket(remote);
        conn.send(&Event::Movement(0, 32.0, 64.0, -128.0));
    }

    fn client_main() -> Result<(), ()> {
        let remote = TcpStream::connect("localhost:1337").unwrap();
        let mut conn = Connection::<DumbProtocol>::from_socket(remote);
        match conn.recv_blocking() {
            Some(Event::Movement(0, 32.0, 64.0, -128.0)) => Ok(()),
            _ => Err(()),
        }
    }

    let server = thread::spawn(server_main);
    let client = thread::spawn(client_main);

    let success = match client.join() {
        Ok(Ok(())) => true,
        _ => false,
    };
    server.join();
    assert!(success);
}

#[test]
fn dumb_protocol_multiple_event_transfer() {
    fn server_main() {
        let listener = TcpListener::bind("localhost:1338").unwrap();
        let (remote, _address) = listener.accept().unwrap();
        let mut conn = Connection::<DumbProtocol>::from_socket(remote);
        conn.send(&Event::Movement(0, 32.0, 64.0, -128.0));
        conn.send(&Event::Movement(0, 33.0, 65.0, 128.0));
    }

    fn client_main() -> Result<(), ()> {
        let remote = TcpStream::connect("localhost:1338").unwrap();
        let mut conn = Connection::<DumbProtocol>::from_socket(remote);
        let first = match conn.recv_blocking() {
            Some(Event::Movement(0, 32.0, 64.0, -128.0)) => Ok(()),
            _ => Err(()),
        };
        let second = match conn.recv_blocking() {
            Some(Event::Movement(0, 33.0, 65.0, 128.0)) => Ok(()),
            _ => Err(()),
        };
        first.and(second)
    }

    let server = thread::spawn(server_main);
    let client = thread::spawn(client_main);

    let success = match client.join() {
        Ok(Ok(())) => true,
        _ => false,
    };
    server.join();
    assert!(success);
}

#[test]
fn smart_protocol_encode_decode_movement() {
    let expected = Event::Movement(0xff, 1432.0, -1432.0, -13452.0);
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_ready() {
    let expected = Event::Ready;
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_start() {
    let expected = Event::Start;
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_yield() {
    let expected = Event::Yield(u64::max_value());
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_request_movement() {
    let expected = Event::RequestMovement(0xff, 1432.0, -1432.0, -13452.0);
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_custom() {
    let expected = Event::Custom(2131231, vec![234, 3, b'\n', 34, b'\x1b', 32, b'\n']);
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_spawn() {
    let expected = Event::Spawn(u64::max_value());
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_key_down() {
    let expected = Event::KeyDown(KeyCode::Apostrophe);
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_key_up() {
    let expected = Event::KeyUp(KeyCode::Calculator);
    test_protocol_encode_decode::<SmartProtocol>(expected);
}

#[test]
fn smart_protocol_encode_decode_key_down_all_keys() {
    for key in &ALL_KEYS {
        let expected = Event::KeyDown(*key);
        test_protocol_encode_decode::<SmartProtocol>(expected);
    }
}

#[test]
fn smart_protocol_encode_decode_key_up_all_keys() {
    for key in &ALL_KEYS {
        let expected = Event::KeyUp(*key);
        test_protocol_encode_decode::<SmartProtocol>(expected);
    }
}

#[test]
fn smart_protocol_single_event_transfer() {
    fn server_main() {
        let listener = TcpListener::bind("localhost:1339").unwrap();
        let (remote, _address) = listener.accept().unwrap();
        let mut conn = Connection::<SmartProtocol>::from_socket(remote);
        conn.send(&Event::Movement(0, 32.0, 64.0, -128.0));
    }

    fn client_main() -> Result<(), ()> {
        let remote = TcpStream::connect("localhost:1339").unwrap();
        let mut conn = Connection::<SmartProtocol>::from_socket(remote);
        match conn.recv_blocking() {
            Some(Event::Movement(0, 32.0, 64.0, -128.0)) => Ok(()),
            _ => Err(()),
        }
    }

    let server = thread::spawn(server_main);
    let client = thread::spawn(client_main);

    let success = match client.join() {
        Ok(Ok(())) => true,
        _ => false,
    };
    server.join();
    assert!(success);
}

#[test]
fn smart_protocol_multiple_event_transfer() {
    fn server_main() {
        let listener = TcpListener::bind("localhost:1340").unwrap();
        let (remote, _address) = listener.accept().unwrap();
        let mut conn = Connection::<SmartProtocol>::from_socket(remote);
        conn.send(&Event::Movement(0, 32.0, 64.0, -128.0));
        conn.send(&Event::Movement(0, 33.0, 65.0, 128.0));
    }

    fn client_main() -> Result<(), ()> {
        let remote = TcpStream::connect("localhost:1340").unwrap();
        let mut conn = Connection::<SmartProtocol>::from_socket(remote);
        let first = match conn.recv_blocking() {
            Some(Event::Movement(0, 32.0, 64.0, -128.0)) => Ok(()),
            _ => Err(()),
        };
        let second = match conn.recv_blocking() {
            Some(Event::Movement(0, 33.0, 65.0, 128.0)) => Ok(()),
            _ => Err(()),
        };
        first.and(second)
    }

    let server = thread::spawn(server_main);
    let client = thread::spawn(client_main);

    let success = match client.join() {
        Ok(Ok(())) => true,
        _ => false,
    };
    server.join();
    assert!(success);
}
