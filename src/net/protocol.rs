#![allow(deprecated, unused)]

use crate::game::graphics::MeshType;
use crate::misc::constants::{ALL_KEYS, ALL_MESH_TYPES, KEY_INDEX_MAP, MESH_INDEX_MAP};
use crate::net::{Event, Handle};
use ggez::event::KeyCode;
use ggez::graphics::Color;
use std::mem::size_of;

pub trait Protocol {
    fn encode(event: &Event) -> Vec<u8>;
    fn decode(bytes: &[u8]) -> Option<Event>;
}

/// A concise protocol which serializes events into a leading bytes signifying variant
/// and a series of trailing bytes containing the data held by an event
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
            Event::Spawn(handle, mesh_type) => Self::encode_spawn(*handle, *mesh_type),
            Event::PickUp(handle, mesh_type) => Self::encode_pick_up(*handle, *mesh_type),
            Event::Despawn(handle) => Self::encode_despawn(*handle),
            Event::KeyDown(key_code) => Self::encode_key_down(*key_code),
            Event::KeyUp(key_code) => Self::encode_key_up(*key_code),
            Event::Health(handle, health) => Self::encode_health(*handle, *health),
            Event::Color(handle, color) => Self::encode_color(*handle, *color),
            Event::Dimension(handle, width, height) => Self::encode_dimension(*handle, *width, *height)
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
            // r is for ready
            b'r' => Self::decode_ready(data),
            // S is for Start
            b'S' => Self::decode_start(data),
            // M is for Movement
            b'M' => Self::decode_movement(data),
            // m is for movement but less authoritative
            b'm' => Self::decode_request_movement(data),
            // c is for custom
            b'c' => Self::decode_custom(data),
            // Y is for Yield
            b'Y' => Self::decode_yield(data),
            // P is for ...sPawn?
            b'P' => Self::decode_spawn(data),
            // p is for pick up
            b'p' => Self::decode_pick_up(data),
            // d is for down
            b'd' => Self::decode_key_down(data),
            // u is for up
            b'u' => Self::decode_key_up(data),
            // B is for Bye!
            b'B' => Self::decode_despawn(data),
            // H is for Health
            b'H' => Self::decode_health(data),
            // C is for Color (this protocol is not to be used within the UK)
            b'C' => Self::decode_color(data),
            // D is for Dimension
            b'D' => Self::decode_dimension(data),
            // _ is for unsupported or invalid
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

    /* Below are all encoding and decoding functions,
     * they're messy, but also pretty straightforward
     * so most won't be commented */

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
        if data.len() == size_of::<Handle>() + size_of::<usize>() {
            let handle = unsigned_from_bytes(&data[..size_of::<Handle>()]) as Handle;
            // Meshes are serialized using an array, look up which index contains this mesh,
            // then send the index
            let mesh_index = unsigned_from_bytes(&data[size_of::<Handle>()..]) as usize;
            let mesh_type = ALL_MESH_TYPES[mesh_index];
            Some(Event::Spawn(handle, mesh_type))
        } else {
            None
        }
    }

    fn encode_spawn(handle: Handle, mesh_type: MeshType) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>() + size_of::<usize>());
        let mesh_index = MESH_INDEX_MAP.get(&mesh_type).expect(&format!(
            "Critical protocol failure. Missing mesh_type {:?} in MESH_INDEX_MAP",
            mesh_type
        ));
        bytes.push(b'P');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.append(&mut usize_to_bytes(*mesh_index));
        bytes
    }

    fn decode_pick_up(data: &[u8]) -> Option<Event> {
        if data.len() == size_of::<Handle>() + size_of::<usize>() {
            let handle = unsigned_from_bytes(&data[..size_of::<Handle>()]) as Handle;
            // Meshes are serialized using an array, look up which index contains this mesh,
            // then send the index
            let mesh_index = unsigned_from_bytes(&data[size_of::<Handle>()..]) as usize;
            let mesh_type = ALL_MESH_TYPES[mesh_index];
            Some(Event::PickUp(handle, mesh_type))
        } else {
            None
        }
    }

    fn encode_pick_up(handle: Handle, mesh_type: MeshType) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>() + size_of::<usize>());
        let mesh_index = MESH_INDEX_MAP.get(&mesh_type).expect(&format!(
            "Critical protocol failure. Missing mesh_type {:?} in MESH_INDEX_MAP",
            mesh_type
        ));
        bytes.push(b'p');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.append(&mut usize_to_bytes(*mesh_index));
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
        // Keys are serialized using an array, look up which index contains this key_code,
        // then send the index
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
        // Keys are serialized using an array, look up which index contains this key_code,
        // then send the index
        let key_index = KEY_INDEX_MAP.get(&key_code).expect(&format!(
            "Critical protocol failure. Missing code {:?} in KEY_INDEX_MAP",
            key_code
        ));
        let mut bytes = Vec::with_capacity(1 + size_of::<usize>());
        bytes.push(b'u');
        bytes.append(&mut usize_to_bytes(*key_index));
        bytes
    }

    fn decode_despawn(data: &[u8]) -> Option<Event> {
        if data.len() == 8 {
            let handle = unsigned_from_bytes(data) as Handle;
            Some(Event::Despawn(handle))
        } else {
            None
        }
    }

    fn encode_despawn(handle: Handle) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>());
        bytes.push(b'B');
        bytes.append(&mut u64_to_bytes(handle));
        bytes
    }

    fn decode_health(data: &[u8]) -> Option<Event> {
        if data.len() == size_of::<Handle>() + size_of::<u8>() {
            let handle = unsigned_from_bytes(&data[..8]) as Handle;
            let health = unsigned_from_bytes(&data[8..]) as u8;
            Some(Event::Health(handle, health))
        } else {
            None
        }
    }

    fn encode_health(handle: Handle, health: u8) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>() + size_of::<u8>());
        bytes.push(b'H');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.push(health);
        bytes
    }

    fn decode_color(data: &[u8]) -> Option<Event> {
        if data.len() == size_of::<Handle>() + 4 * size_of::<u8>() {
            let handle = unsigned_from_bytes(&data[..8]) as Handle;
            let r = (data[8] as f32) / 255.0;
            let g = (data[9] as f32) / 255.0;
            let b = (data[10] as f32) / 255.0;
            let a = (data[11] as f32) / 255.0;
            Some(Event::Color(handle, Color::new(r, g, b, a)))
        } else {
            None
        }
    }

    fn encode_color(handle: Handle, color: Color) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(1 + size_of::<Handle>() + 4 * size_of::<u8>());
        bytes.push(b'C');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.push((color.r * 255.0) as u8);
        bytes.push((color.g * 255.0) as u8);
        bytes.push((color.b * 255.0) as u8);
        bytes.push((color.a * 255.0) as u8);
        bytes
    }

    fn decode_dimension(data: &[u8]) -> Option<Event> {
        const HANDLE_SIZE: usize = size_of::<Handle>();
        const DIM_SIZE: usize = size_of::<f32>();
        const EXPECTED_LENGTH: usize = HANDLE_SIZE + 2 * DIM_SIZE;
        if data.len() == EXPECTED_LENGTH {
            let handle = unsigned_from_bytes(&data[..HANDLE_SIZE]) as Handle;
            let width = f32_from_bytes(&data[HANDLE_SIZE..HANDLE_SIZE + DIM_SIZE]);
            let height = f32_from_bytes(&data[HANDLE_SIZE + DIM_SIZE..HANDLE_SIZE + DIM_SIZE * 2]);
            Some(Event::Dimension(handle, width, height))
        } else {
            None
        }
    }

    fn encode_dimension(handle: Handle, width: f32, height: f32) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(b'D');
        bytes.append(&mut u64_to_bytes(handle));
        bytes.append(&mut f32_to_bytes(width));
        bytes.append(&mut f32_to_bytes(height));
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
