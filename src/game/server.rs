use crate::game::ecs::{prefabs, ControlSystem, Entity, PositionWatcherSystem, System};
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::event::KeyCode;
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

const CLIENT_COUNT: usize = 2;

pub struct Server<PROTOCOL: Protocol> {
    clients: [Connection<PROTOCOL>; CLIENT_COUNT],
    handles: HashSet<Handle>,
    // The latest handle to be assigned
    last_handle: Handle,
    // Keys currently held down for each client
    pressed_keys: [HashSet<KeyCode>; CLIENT_COUNT],
    systems: Vec<Box<dyn System>>,
    position_watcher: PositionWatcherSystem,
    entities: Vec<Entity>,
    events: VecDeque<Event>,
    delta_time: f32,
}

impl<PROTOCOL: Protocol> Server<PROTOCOL> {
    pub fn main(&mut self) {
        self.await_clients();
        self.spawn_players();
        let mut last_frame = Instant::now();
        let mut last_broadcast = Instant::now();
        const MIN_BROADCAST_DURATION: Duration = Duration::from_micros(0);
        const MIN_FRAME_DURATION: Duration = Duration::from_millis(20);
        loop {
            for i in 0..self.clients.len() {
                if let Some(event) = self.clients[i].recv() {
                    self.handle(i, event);
                }
            }
            if last_frame.elapsed() >= MIN_FRAME_DURATION {
                self.delta_time = last_frame.elapsed().as_secs_f32();
                last_frame = Instant::now();
                let mut new_events = self.call_systems();
                self.events.append(&mut new_events);
                if last_broadcast.elapsed() >= MIN_BROADCAST_DURATION {
                    // todo: reason about whether this line should come before or after broadcasting
                    last_broadcast = Instant::now();
                    // todo: new_events and movement_events should come in the right order
                    // this will require some ecs overhauling
                    let mut movement_events = self.get_movement_events();
                    self.events.append(&mut movement_events);
                    while !self.events.is_empty() {
                        if let Some(event) = self.events.pop_front() {
                            self.broadcast_event(&event)
                        }
                    }
                }
            }
        }
    }

    pub fn new(client1: Connection<PROTOCOL>, client2: Connection<PROTOCOL>) -> Self {
        let clients = [client1, client2];
        Self {
            clients,
            handles: Default::default(),
            last_handle: NULL_HANDLE,
            pressed_keys: [HashSet::new(), HashSet::new()],
            systems: vec![Box::new(ControlSystem)],
            position_watcher: PositionWatcherSystem,
            entities: vec![],
            events: VecDeque::new(),
            delta_time: 0.0,
        }
    }

    fn call_systems(&mut self) -> VecDeque<Event> {
        // todo: maybe don't clone the keys each time
        let mut ctx = ServerContext::new(self.pressed_keys.clone(), self.delta_time);
        for system in &mut self.systems {
            system.update(&mut self.entities, &mut ctx);
        }
        ctx.take_events()
    }

    fn get_movement_events(&mut self) -> VecDeque<Event> {
        let mut ctx = ServerContext::new(self.pressed_keys.clone(), self.delta_time);
        self.position_watcher.update(&mut self.entities, &mut ctx);
        ctx.take_events()
    }

    fn spawn_players(&mut self) {
        for i in 0..self.clients.len() {
            self.spawn();
            self.entities
                .push(prefabs::player(self.last_handle, i, 0.0, 0.0, 0.0));
        }
    }

    fn spawn(&mut self) {
        let handle = self.last_handle + 1;
        self.handles.insert(handle);
        self.broadcast_event(&Event::Spawn(handle));
        self.last_handle = handle;
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
        self.pressed_keys[conn_index].remove(&key_code);
    }

    fn on_key_down(&mut self, conn_index: usize, key_code: KeyCode) {
        self.pressed_keys[conn_index].insert(key_code);
    }
}

pub struct ServerContext {
    // HashSets of keys pressed on each client
    input_devices: [HashSet<KeyCode>; CLIENT_COUNT],
    delta_time: f32,
    events: VecDeque<Event>,
}

impl ServerContext {
    fn new(input_devices: [HashSet<KeyCode>; CLIENT_COUNT], delta_time: f32) -> Self {
        Self {
            input_devices,
            delta_time,
            events: VecDeque::new(),
        }
    }

    pub fn pressed_keys(&self, input_device_index: usize) -> &HashSet<KeyCode> {
        &self.input_devices[input_device_index]
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    fn take_events(self) -> VecDeque<Event> {
        self.events
    }
}
