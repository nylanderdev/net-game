use crate::net::{Event, Protocol};
use std::collections::VecDeque;
use std::io::{Read, Result as IOResult, Write};
use std::marker::PhantomData;
use std::net::TcpStream;

/// An enum representing various types of endpoints
/// that can be used to send serialized data
// Only sockets (TcpStream) are currently supported
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

// The read trait is used to receive bytes. It is provided by the standard lib.
impl Read for Endpoint {
    fn read(&mut self, buf: &mut [u8]) -> IOResult<usize> {
        match self {
            // Propagate to actual socket
            Endpoint::Socket(socket) => socket.read(buf),
        }
    }
}

// The read trait is used to send bytes. It is provided by the standard lib.
impl Write for Endpoint {
    fn write(&mut self, buf: &[u8]) -> IOResult<usize> {
        match self {
            // Propagate to actual socket
            Endpoint::Socket(socket) => socket.write(buf),
        }
    }

    fn flush(&mut self) -> IOResult<()> {
        match self {
            // Propagate to actual socket
            Endpoint::Socket(socket) => socket.flush(),
        }
    }
}

pub struct Connection<PROTOCOL: Protocol> {
    endpoint: Endpoint,
    // A marker which allows us to use the PROTOCOL generic without the compiler complaining
    protocol_marker: PhantomData<PROTOCOL>,
}

impl<PROTOCOL: Protocol> Connection<PROTOCOL> {
    pub fn from_socket(socket: TcpStream) -> Self {
        // Don't block on sending or receiving
        socket.set_nonblocking(true);
        // Send data immediately, otherwise game turns laggy
        socket.set_nodelay(true);
        let endpoint = Endpoint::Socket(socket);
        Self {
            endpoint,
            protocol_marker: PhantomData::<PROTOCOL>,
        }
    }
    /// Send an event
    pub fn send(&mut self, event: &Event) {
        let event_bytes = PROTOCOL::encode(&event);
        // Escape any reserved bytes (newline and ESC)
        let mut escaped_bytes = escape_bytes(&event_bytes);
        // Separate serialized events with a newline
        // this is why we escape bytes
        escaped_bytes.push('\n' as u8);
        // todo: proper error handling
        self.endpoint.write(escaped_bytes.as_slice());
    }
    /// Receive an event, if there are any available
    pub fn recv(&mut self) -> Option<Event> {
        // todo: reason about this limit
        const BUFFER_LEN: usize = 1024;
        let mut buffer = [0; BUFFER_LEN];
        // Peek bytes instead of reading them,
        // as we don't want to remove them from
        // the socket buffer unless we've received
        // enough bytes to assemble an event
        self.endpoint.peek(&mut buffer).ok()?;
        // Get the index of the first (unescaped) newline, i.e the end of the event's bytes
        let first_newline_index = index_of_first_unescaped_newline(&buffer)?;
        let escaped_byte_count = first_newline_index + 1;
        let mut escaped_bytes = vec![0; escaped_byte_count];
        // Only read the bytes that are part of the event
        self.endpoint.read_exact(escaped_bytes.as_mut_slice());
        // Remove trailing newline and unescape escaped characters
        let event_bytes = unescape_bytes(&escaped_bytes[..escaped_bytes.len() - 1]);
        // Let the protocol interpret bytes and deserialize into an event
        PROTOCOL::decode(&event_bytes)
    }
    #[cfg(test)] // Only used in tests, for now
    /// Block until an event has been received
    pub fn recv_blocking(&mut self) -> Option<Event> {
        // Just spin until there's something available
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
                // No more events for the moment, time to return what we've got so far
                break;
            }
        }
        events
    }
    /// Returns whether this connection is on this machine
    pub fn is_local(&self) -> bool {
        match &self.endpoint {
            Endpoint::Socket(socket) => socket.peer_addr().unwrap().ip().is_loopback()
        }
    }
}

/// Escape reserve bytes ('\n' and '\x1b') by placing the byte '\x1b' before them
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
            // All out of bytes, time for a break
            None => break,
        }
    }
    escaped
}

/// Undo any previous escaping of bytes. DO NOT call on unescaped bytes
fn unescape_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut iter = bytes.iter();
    let mut unescaped = Vec::new();
    loop {
        match iter.next() {
            // ESC
            Some(b'\x1b') => {
                // Use iter.next() to skip the escape byte ('\x1b')
                // and get right to the good stuff
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
                // This is an escaped byte, skip it
                iter.next();
            }
            (index, b'\n') => return Some(index),
            // For all other bytes just do nothing
            _ => (),
        }
    }
}
