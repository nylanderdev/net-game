#![allow(deprecated, unused)]

use crate::misc::constants::{ALL_KEYS, KEY_INDEX_MAP};
use crate::net::{Event, Handle};
use ggez::event::KeyCode;
use std::mem::size_of;

pub trait Protocol {
    fn encode(event: &Event) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Event>;
}

#[deprecated]
pub struct DumbProtocol;

impl Protocol for DumbProtocol {
    fn encode(event: &Event) -> Vec<u8> {
        let code_string = format!("{:?}", event);
        code_string.into_bytes()
    }

    fn decode(bytes: &[u8]) -> Option<Event> {
        let byte_vec = Vec::from(bytes);
        let mut string = String::from_utf8(byte_vec).ok()?;
        string.retain(|c| !c.is_whitespace());
        let first_paren = string.find('(');
        let last_paren = string.find(')');
        let event = if first_paren.is_some()
            && last_paren.is_some()
            && last_paren.unwrap() == string.len() - 1
        {
            // Tuple struct
            let enum_name = &string[..first_paren.unwrap()];
            let contents = &string[first_paren.unwrap() + 1..last_paren.unwrap()];
            match enum_name {
                "Movement" => {
                    let coords: Vec<f32> =
                        contents.split(",").filter_map(|s| s.parse().ok()).collect();
                    if coords.len() == 4 {
                        Some(Event::Movement(
                            coords[0] as u64,
                            coords[1],
                            coords[2],
                            coords[3],
                        ))
                    } else {
                        None
                    }
                }
                "RequestMovement" => {
                    let mut split_contents = contents.split(",");
                    let handle = split_contents.next()?.parse::<Handle>().ok()?;
                    let coords: Vec<f32> = split_contents.filter_map(|s| s.parse().ok()).collect();
                    if coords.len() == 3 {
                        Some(Event::RequestMovement(
                            handle, coords[0], coords[1], coords[2],
                        ))
                    } else {
                        None
                    }
                }
                "Yield" => {
                    let parsed = contents.parse::<Handle>().ok()?;
                    Some(Event::Yield(parsed))
                }
                "Spawn" => {
                    let parsed = contents.parse::<Handle>().ok()?;
                    Some(Event::Spawn(parsed))
                }
                "Custom" => {
                    // Filter out brackets from the vec
                    let numbers = contents.replace("[", "").replace("]", "");
                    let mut split_numbers = numbers.split(",");
                    let kind = split_numbers.next()?.parse::<u32>().ok()?;
                    let bytes: Vec<u8> = split_numbers.filter_map(|s| s.parse().ok()).collect();
                    Some(Event::Custom(kind, bytes))
                }
                _ => None,
            }
        } else {
            match &string[..] {
                "Ready" => Some(Event::Ready),
                "Start" => Some(Event::Start),
                _ => None,
            }
        };
        event
    }
}

pub struct SmartProtocol;

impl Protocol for SmartProtocol {
    fn encode(event: &Event) -> Vec<u8> {
        match event {
            Event::Ready => Self::encode_ready(),
            Event::Start => Self::encode_start(),
            Event::Movement(handle, x, y, angle) => Self::encode_movement(*handle, *x, *y, *angle),
            Event::RequestMovement(handle, x, y, angle) => {
                Self::encode_request_movement(*handle, *x, *y, *angle)
            }
            Event::Custom(kind, data) => Self::encode_custom(*kind, data),
            Event::Yield(handle) => Self::encode_yield(*handle),
            Event::Spawn(handle) => Self::encode_spawn(*handle),
            Event::KeyDown(key_code) => Self::encode_key_down(*key_code),
            Event::KeyUp(key_code) => Self::encode_key_up(*key_code),
        }
    }

    fn decode(bytes: &[u8]) -> Option<Event> {
        if bytes.is_empty() {
            None
        } else {
            let leading_byte = bytes[0];
            Self::interpret_data(leading_byte, &bytes[1..])
        }
    }
}

impl SmartProtocol {
    fn interpret_data(leading_byte: u8, data: &[u8]) -> Option<Event> {
        match leading_byte {
            b'r' => Self::decode_ready(data),
            b'S' => Self::decode_start(data),
            b'M' => Self::decode_movement(data),
            b'm' => Self::decode_request_movement(data),
            b'c' => Self::decode_custom(data),
            b'Y' => Self::decode_yield(data),
            b'P' => Self::decode_spawn(data),
            b'd' => Self::decode_key_down(data),
            b'u' => Self::decode_key_up(data),
            _ => None,
        }
    }

    fn decode_ready(data: &[u8]) -> Option<Event> {
        if data.is_empty() {
            Some(Event::Ready)
        } else {
            None
        }
    }

    fn encode_ready() -> Vec<u8> {
        vec![b'r']
    }

    fn decode_start(data: &[u8]) -> Option<Event> {
        if data.is_empty() {
            Some(Event::Start)
        } else {
            None
        }
    }

    fn encode_start() -> Vec<u8> {
        vec![b'S']
    }

    fn decode_movement(data: &[u8]) -> Option<Event> {
        const HANDLE_SIZE: usize = size_of::<Handle>();
        const COORD_SIZE: usize = size_of::<f32>();
        const EXPECTED_LENGTH: usize = HANDLE_SIZE + 3 * COORD_SIZE;
        if data.len() == EXPECTED_LENGTH {
            let handle = unsigned_from_bytes(&data[..HANDLE_SIZE]) as Handle;
            let x = f32_from_bytes(&data[HANDLE_SIZE..HANDLE_SIZE + COORD_SIZE]) as f32;
            let y = f32_from_bytes(&data[HANDLE_SIZE + COORD_SIZE..HANDLE_SIZE + COORD_SIZE * 2])
                as f32;
            let angle = f32_from_bytes(&data[HANDLE_SIZE + COORD_SIZE * 2..EXPECTED_LENGTH]) as f32;
            Some(Event::Movement(handle, x, y, angle))
        } else {
            None
        }
    }

    fn encode_movement(handle: Handle, x: f32, y: f32, angle: f32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + 8 + 4 + 4);
        bytes.push(b'M');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.append(&mut f32_to_bytes(x));
        bytes.append(&mut f32_to_bytes(y));
        bytes.append(&mut f32_to_bytes(angle));
        bytes
    }

    fn decode_request_movement(data: &[u8]) -> Option<Event> {
        const HANDLE_SIZE: usize = size_of::<Handle>();
        const COORD_SIZE: usize = size_of::<f32>();
        const EXPECTED_LENGTH: usize = HANDLE_SIZE + 3 * COORD_SIZE;
        if data.len() == EXPECTED_LENGTH {
            let handle = unsigned_from_bytes(&data[..HANDLE_SIZE]) as Handle;
            let x = f32_from_bytes(&data[HANDLE_SIZE..HANDLE_SIZE + COORD_SIZE]) as f32;
            let y = f32_from_bytes(&data[HANDLE_SIZE + COORD_SIZE..HANDLE_SIZE + COORD_SIZE * 2])
                as f32;
            let angle = f32_from_bytes(&data[HANDLE_SIZE + COORD_SIZE * 2..EXPECTED_LENGTH]) as f32;
            Some(Event::RequestMovement(handle, x, y, angle))
        } else {
            None
        }
    }

    fn encode_request_movement(handle: Handle, x: f32, y: f32, angle: f32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + 8 + 4 + 4);
        bytes.push(b'm');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.append(&mut f32_to_bytes(x));
        bytes.append(&mut f32_to_bytes(y));
        bytes.append(&mut f32_to_bytes(angle));
        bytes
    }

    fn decode_custom(data: &[u8]) -> Option<Event> {
        if !data.is_empty() {
            let kind = unsigned_from_bytes(&data[..4]) as u32;
            let custom_data = data[4..].iter().map(|byte| *byte).collect();
            Some(Event::Custom(kind, custom_data))
        } else {
            None
        }
    }

    fn encode_custom(kind: u32, data: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + 4 + data.len());
        bytes.push(b'c');
        bytes.append(&mut u32_to_bytes(kind));
        bytes.extend_from_slice(data);
        bytes
    }

    fn decode_yield(data: &[u8]) -> Option<Event> {
        if data.len() == 8 {
            let handle = unsigned_from_bytes(data) as Handle;
            Some(Event::Yield(handle))
        } else {
            None
        }
    }

    fn encode_yield(handle: Handle) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>());
        bytes.push(b'Y');
        bytes.append(&mut u64_to_bytes(handle));
        bytes
    }

    fn decode_spawn(data: &[u8]) -> Option<Event> {
        if data.len() == 8 {
            let handle = unsigned_from_bytes(data) as Handle;
            Some(Event::Spawn(handle))
        } else {
            None
        }
    }

    fn encode_spawn(handle: Handle) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>());
        bytes.push(b'P');
        bytes.append(&mut u64_to_bytes(handle));
        bytes
    }

    fn decode_key_down(data: &[u8]) -> Option<Event> {
        if data.len() == size_of::<usize>() {
            let key_index = unsigned_from_bytes(data) as usize;
            let key_code = ALL_KEYS[key_index];
            Some(Event::KeyDown(key_code))
        } else {
            None
        }
    }

    fn encode_key_down(key_code: KeyCode) -> Vec<u8> {
        let key_index = KEY_INDEX_MAP.get(&key_code).expect(&format!(
            "Critical protocol failure. Missing code {:?} in KEY_INDEX_MAP",
            key_code
        ));
        let mut bytes = Vec::with_capacity(1 + size_of::<usize>());
        bytes.push(b'd');
        bytes.append(&mut usize_to_bytes(*key_index));
        bytes
    }

    fn decode_key_up(data: &[u8]) -> Option<Event> {
        if data.len() == size_of::<usize>() {
            let key_index = unsigned_from_bytes(data) as usize;
            let key_code = ALL_KEYS[key_index];
            Some(Event::KeyUp(key_code))
        } else {
            None
        }
    }

    fn encode_key_up(key_code: KeyCode) -> Vec<u8> {
        let key_index = KEY_INDEX_MAP.get(&key_code).expect(&format!(
            "Critical protocol failure. Missing code {:?} in KEY_INDEX_MAP",
            key_code
        ));
        let mut bytes = Vec::with_capacity(1 + size_of::<usize>());
        bytes.push(b'u');
        bytes.append(&mut usize_to_bytes(*key_index));
        bytes
    }
}

fn u32_to_bytes(number: u32) -> Vec<u8> {
    number.to_be_bytes().to_vec()
}

fn usize_to_bytes(number: usize) -> Vec<u8> {
    number.to_be_bytes().to_vec()
}

fn u64_to_bytes(number: u64) -> Vec<u8> {
    number.to_be_bytes().to_vec()
}

fn i32_to_bytes(number: i32) -> Vec<u8> {
    number.to_be_bytes().to_vec()
}

fn f32_to_bytes(number: f32) -> Vec<u8> {
    number.to_be_bytes().to_vec()
}

fn unsigned_from_bytes(bytes: &[u8]) -> u128 {
    let mut unsigned = 0;
    for byte in bytes {
        unsigned <<= 8;
        unsigned += *byte as u128;
    }
    unsigned
}

fn i32_from_bytes(bytes: &[u8]) -> i32 {
    let mut four_bytes = [0; 4];
    for i in 0..bytes.len().min(4) {
        four_bytes[i] = bytes[i]
    }
    i32::from_be_bytes(four_bytes)
}

fn f32_from_bytes(bytes: &[u8]) -> f32 {
    let mut four_bytes = [0; 4];
    for i in 0..bytes.len().min(4) {
        four_bytes[i] = bytes[i]
    }
    f32::from_be_bytes(four_bytes)
}
