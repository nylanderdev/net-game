use crate::net::{Event, Protocol};
use std::collections::VecDeque;
use std::io::{Read, Result as IOResult, Write};
use std::marker::PhantomData;
use std::net::TcpStream;

enum Endpoint {
    Socket(TcpStream),
}

impl Endpoint {
    pub fn peek(&self, buf: &mut [u8]) -> IOResult<usize> {
        match self {
            Endpoint::Socket(socket) => socket.peek(buf),
        }
    }
}

impl Read for Endpoint {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        match self {
            Endpoint::Socket(socket) => socket.read(buf),
        }
    }
}

impl Write for Endpoint {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        match self {
            Endpoint::Socket(socket) => socket.write(buf),
        }
    }

    fn flush(&mut self) -> IOResult<()> {
        match self {
            Endpoint::Socket(socket) => socket.flush(),
        }
    }
}

pub struct Connection<PROTOCOL: Protocol> {
    endpoint: Endpoint,
    protocol_marker: PhantomData<PROTOCOL>,
}

impl<PROTOCOL: Protocol> Connection<PROTOCOL> {
    pub fn from_socket(socket: TcpStream) -> Self {
        socket.set_nonblocking(true);
        let endpoint = Endpoint::Socket(socket);
        Self {
            endpoint,
            protocol_marker: PhantomData::<PROTOCOL>,
        }
    }
    pub fn send(&mut self, event: &Event) {
        let event_bytes = PROTOCOL::encode(&event);
        let mut escaped_bytes = escape_bytes(&event_bytes);
        escaped_bytes.push('\n' as u8);
        // todo: proper error handling
        self.endpoint.write(escaped_bytes.as_slice());
    }
    pub fn recv(&mut self) -> Option<Event> {
        // todo: reason about this limit
        const BUFFER_LEN: usize = 1024;
        let mut buffer = [0; BUFFER_LEN];
        self.endpoint.peek(&mut buffer).ok()?;
        let first_newline_index = index_of_first_unescaped_newline(&buffer)?;
        let escaped_byte_count = first_newline_index + 1;
        let mut escaped_bytes = vec![0; escaped_byte_count];
        self.endpoint.read_exact(escaped_bytes.as_mut_slice());
        // Without trailing newline or escaped characters
        let event_bytes = unescape_bytes(&escaped_bytes[..escaped_bytes.len() - 1]);
        PROTOCOL::decode(&event_bytes)
    }
    pub fn recv_blocking(&mut self) -> Option<Event> {
        loop {
            let option = self.recv();
            if option.is_some() {
                return option;
            }
        }
    }
    /// Receive up to event_limit events
    pub fn recv_multiple(&mut self, event_limit: usize) -> VecDeque<Event> {
        let mut events = VecDeque::with_capacity(event_limit);
        while events.len() < event_limit {
            if let Some(event) = self.recv() {
                events.push_back(event);
            } else {
                break;
            }
        }
        events
    }
}

fn escape_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut iter = bytes.iter();
    let mut escaped = Vec::new();
    loop {
        match iter.next() {
            // ESC
            Some(b'\x1b') => {
                escaped.push(b'\x1b');
                escaped.push(b'\x1b');
            }
            // Newline
            Some(b'\n') => {
                escaped.push(b'\x1b');
                escaped.push(b'\n');
            }
            Some(byte) => escaped.push(*byte),
            None => break,
        }
    }
    escaped
}

fn unescape_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut iter = bytes.iter();
    let mut unescaped = Vec::new();
    loop {
        match iter.next() {
            // ESC
            Some(b'\x1b') => {
                if let Some(escaped_byte) = iter.next() {
                    unescaped.push(*escaped_byte);
                }
            }
            Some(byte) => unescaped.push(*byte),
            None => break,
        }
    }
    unescaped
}

fn index_of_first_unescaped_newline(bytes: &[u8]) -> Option<usize> {
    let mut iter = bytes.iter().enumerate();
    loop {
        match iter.next()? {
            // ESC
            (_, b'\x1b') => {
                iter.next();
            }
            (index, b'\n') => return Some(index),
            _ => (),
        }
    }
}
