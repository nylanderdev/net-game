use crate::net::{Connection, Event, EventListener, Handle, Protocol};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};
use std::thread::sleep;

pub struct Server<PROTOCOL: Protocol> {
    clients: [Connection<PROTOCOL>; 2],
    owned_handles: HashMap<usize, Vec<Handle>>,
    coords: HashMap<Handle, (i32, i32)>,
    movement_map: HashMap<Handle, HashSet<Direction>>,
}

#[derive(Eq, PartialEq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl<PROTOCOL: Protocol> Server<PROTOCOL> {
    pub fn main(&mut self) {
        self.await_clients();
        let mut last_frame = Instant::now();
        loop {
            const FRAME_DURATION: Duration = Duration::from_millis(10);
            const HANDLED_EVENT_COUNT: usize = 4;
            for _count in 0..HANDLED_EVENT_COUNT {
                for i in 0..self.clients.len() {
                    if let Some(event) = self.clients[i].recv() {
                        if self.validate_event(&event, i) {
                            println!("{:?}", event);
                            self.handle(event);
                        }
                    }
                }
            }
            if last_frame.elapsed() >= FRAME_DURATION {
                last_frame = Instant::now();
                self.server_frame();
            }
        }
    }

    fn handle_movement(&mut self) {
        let mut events = VecDeque::new();
        let mut handles = Vec::new();
        for (handle, directions) in &self.movement_map {
            handles.push(*handle);
            let (mut move_x, mut move_y) = (0, 0);
            for direction in directions {
                match direction {
                    Direction::Up => move_y -= 1,
                    Direction::Down => move_y += 1,
                    Direction::Left => move_x -= 1,
                    Direction::Right => move_x += 1,
                    _ => ()
                }
            }
            if let Some((x, y)) = self.coords.get(handle) {
                let new_coords = (x + move_x, y + move_y);
                self.coords.insert(*handle, new_coords);
                events.push_back(Event::Movement(*handle, new_coords.0, new_coords.1));
            }
        }
        while !events.is_empty() {
            if let Some(event) = events.pop_front() {
                self.broadcast_event(&event);
            }
        }
        for handle in handles {
            self.movement_map.get_mut(&handle).unwrap().clear();
        }
    }

    pub fn new(client1: Connection<PROTOCOL>, client2: Connection<PROTOCOL>) -> Self {
        let clients = [client1, client2];
        let mut owned_handles = HashMap::new();
        // Initialize owned_handles with empty vectors
        for i in 0..clients.len() {
            owned_handles.insert(i, Vec::new());
        }
        Self {
            clients,
            owned_handles,
            coords: HashMap::new(),
            movement_map: HashMap::new(),
        }
    }

    fn await_clients(&mut self) {
        let mut ready = [false, false];
        while !ready[0] || !ready[1] {
            if let Some(Event::Ready) = self.clients[0].recv() {
                ready[0] = true;
            }
            if let Some(Event::Ready) = self.clients[1].recv() {
                ready[1] = true;
            }
        }
        self.broadcast_event(&Event::Start);
        for i in 0..self.clients.len() {
            let player_object_handle = (i + 1) as u64;
            self.owned_handles
                .get_mut(&i)
                .unwrap()
                .push(player_object_handle);
            self.clients[i].send(&Event::Yield(player_object_handle));
            self.broadcast_event(&Event::Spawn(player_object_handle));
        }
    }

    fn recv_input(&mut self) {
        for i in 0..self.clients.len() {
            if let Some(event) = self.clients[i].recv() {
                if self.validate_event(&event, i) {
                    self.handle(event);
                }
            }
        }
    }

    fn broadcast_event(&mut self, event: &Event) {
        self.clients[0].send(event);
        self.clients[1].send(event);
    }

    fn validate_event(&self, event: &Event, client_index: usize) -> bool {
        match event {
            Event::Start => false,
            Event::Movement(..) => false,
            Event::RequestMovement(handle, ..) => {
                client_index < self.clients.len()
                    && self.owned_handles[&client_index].contains(handle)
            }
            Event::Yield(..) => false,
            Event::Spawn(..) => false,
            _ => true,
        }
    }

    fn server_frame(&mut self) {
        let mut events_to_broadcast = Vec::new();
        let coords_copy = self.coords.clone();
        for (handle, (x, y)) in &coords_copy {
            if self.movement_map.get(handle).is_some() {
                const MOVEMENT: i32 = 5;
                let (mut move_x, mut move_y) = (0, 0);
                if self.movement_map[handle].contains(&Direction::Up) { move_y -= MOVEMENT; }
                if self.movement_map[handle].contains(&Direction::Down) { move_y += MOVEMENT; }
                if self.movement_map[handle].contains(&Direction::Left) { move_x -= MOVEMENT; }
                if self.movement_map[handle].contains(&Direction::Right) { move_x += MOVEMENT; }
                if (move_x, move_y) != (0, 0) {
                    events_to_broadcast.push(Event::Movement(*handle, x + move_x, y + move_y));
                }
                self.movement_map.get_mut(handle).unwrap().clear();
                self.coords.insert(*handle, (*x + move_x, *y + move_y));
            } else {
                self.movement_map.insert(*handle, HashSet::new());
            }
        }
        for event in events_to_broadcast {
            self.broadcast_event(&event);
        }
    }
}

impl<PROTOCOL: Protocol> EventListener for Server<PROTOCOL> {
    fn on_request_movement(&mut self, handle: Handle, mut x: i32, mut y: i32) {
        // todo: Movement request should be validated. Consider changing the event struct
        //eprintln!("move on {}: {}, {}", handle, x, y);
        //if is_within_bounds(x, y) {
        if !self.coords.contains_key(&handle) {
            self.coords.insert(handle, (0, 0));
        }
        if self.movement_map.get(&handle).is_none() {
            self.movement_map.insert(handle, HashSet::new());
        }
        if x > 0 {
            x = 1;
            self.movement_map.get_mut(&handle).unwrap().insert(Direction::Right);
        } else if x < 0 {
            x = -1;
            self.movement_map.get_mut(&handle).unwrap().insert(Direction::Left);
        }
        if y > 0 {
            y = 1;
            self.movement_map.get_mut(&handle).unwrap().insert(Direction::Down);
        } else if y < 0 {
            y = -1;
            // todo: do not unwrap
            self.movement_map.get_mut(&handle).unwrap().insert(Direction::Up);
        }
        /*
        if let Some(coords) = self.coords.get(&handle) {
            x *= 10;
            y *= 10;
            x += coords.0;
            y += coords.1;
            self.coords.insert(handle, (x, y));
            self.broadcast_event(&Event::Movement(handle, x, y));
        }

         */
        //}
    }
}

fn is_within_bounds(a: i32, b: i32) -> bool {
    a > -10 && a < 380 && b > -10 && b < 380
}
