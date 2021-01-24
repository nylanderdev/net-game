use crate::misc::State;
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::event::KeyCode;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

const CLIENT_COUNT: usize = 2;

pub struct Server<PROTOCOL: Protocol> {
    clients: [Connection<PROTOCOL>; CLIENT_COUNT],
    owned_handles: [Vec<Handle>; CLIENT_COUNT],
    player_handles: [Handle; CLIENT_COUNT],
    handles: HashSet<Handle>,
    velocity: HashMap<Handle, (f32, f32)>,
    position: HashMap<Handle, State<(f32, f32, f32)>>,
    last_handle: Handle,
    delta_time: f32,
}

impl<PROTOCOL: Protocol> Server<PROTOCOL> {
    pub fn main(&mut self) {
        self.await_clients();
        self.spawn_players();
        let mut last_frame = Instant::now();
        let mut last_broadcast = Instant::now();
        const MIN_BROADCAST_DURATION: Duration = Duration::from_micros(100);
        loop {
            for i in 0..self.clients.len() {
                if let Some(event) = self.clients[i].recv() {
                    self.handle(i, event);
                }
            }
            self.delta_time = last_frame.elapsed().as_secs_f32();
            self.apply_physics();
            last_frame = Instant::now();
            if last_broadcast.elapsed() >= MIN_BROADCAST_DURATION {
                self.send_position();
                last_broadcast = Instant::now();
            }
        }
    }

    fn spawn_players(&mut self) {
        for i in 0..self.clients.len() {
            self.spawn();
            self.owned_handles[i].push(self.last_handle);
            self.player_handles[i] = self.last_handle;
            self.clients[i].send(&Event::Yield(self.last_handle));
        }
    }

    fn spawn(&mut self) {
        let handle = self.last_handle + 1;
        self.handles.insert(handle);
        self.position.insert(handle, State::new((0.0, 0.0, 0.0)));
        self.broadcast_event(&Event::Spawn(handle));
        self.last_handle = handle;
    }

    fn apply_physics(&mut self) {
        for handle in &self.handles {
            let vel = if let Some((x, y)) = self.velocity.get(handle) {
                (*x, *y)
            } else {
                (0.0, 0.0)
            };
            if let Some(pos) = self.position.get_mut(handle) {
                if vel != (0.0, 0.0) {
                    //println!("{:?} : {}", **pos, self.delta_time.log10().round() as isize);
                    pos.0 += vel.0 * self.delta_time;
                    pos.1 += vel.1 * self.delta_time;
                }
            }
        }
    }

    fn send_position(&mut self) {
        let mut events = Vec::with_capacity(self.position.len());
        for (handle, pos) in &mut self.position {
            if pos.invalidated_since() {
                events.push(Event::Movement(*handle, pos.0, pos.1, pos.2));
            }
        }
        for event in events {
            self.broadcast_event(&event);
        }
    }

    pub fn new(client1: Connection<PROTOCOL>, client2: Connection<PROTOCOL>) -> Self {
        let clients = [client1, client2];
        Self {
            clients,
            owned_handles: [vec![], vec![]],
            player_handles: [NULL_HANDLE, NULL_HANDLE],
            handles: Default::default(),
            velocity: HashMap::new(),
            position: HashMap::new(),
            last_handle: 0,
            delta_time: 0.0,
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
    }

    fn broadcast_event(&mut self, event: &Event) {
        for client in &mut self.clients {
            client.send(event);
        }
    }
}

impl<PROTOCOL: Protocol> EventListener for Server<PROTOCOL> {
    fn on_key_up(&mut self, conn_index: usize, key_code: KeyCode) {
        let player_handle = self.player_handles[conn_index];
        let vel = if let Some(vel) = self.velocity.get_mut(&player_handle) {
            vel
        } else {
            self.velocity.insert(player_handle, (0.0, 0.0));
            self.velocity.get_mut(&player_handle).unwrap()
        };
        const VELOCITY: f32 = -200.0;
        match key_code {
            KeyCode::Up => vel.1 -= VELOCITY,
            KeyCode::Down => vel.1 += VELOCITY,
            KeyCode::Left => vel.0 -= VELOCITY,
            KeyCode::Right => vel.0 += VELOCITY,
            _ => (),
        }
    }

    fn on_key_down(&mut self, conn_index: usize, key_code: KeyCode) {
        let player_handle = self.player_handles[conn_index];
        let vel = if let Some(vel) = self.velocity.get_mut(&player_handle) {
            vel
        } else {
            self.velocity.insert(player_handle, (0.0, 0.0));
            self.velocity.get_mut(&player_handle).unwrap()
        };
        const VELOCITY: f32 = 200.0;
        match key_code {
            KeyCode::Up => vel.1 -= VELOCITY,
            KeyCode::Down => vel.1 += VELOCITY,
            KeyCode::Left => vel.0 -= VELOCITY,
            KeyCode::Right => vel.0 += VELOCITY,
            _ => (),
        }
    }
}
