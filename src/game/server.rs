use crate::game::ecs::{prefabs, ColorSystem, ControlSystem, Entity, HealthSystem, PositionWatcherSystem, System, TtlSystem, VelocitySystem, CollisionSystem, ReaperSystem, ScaleSystem, InventorySystem};
use crate::game::graphics::MeshType;
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::event::KeyCode;
use ggez::graphics::{Color, WHITE};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};
use crate::misc::constants::DEFAULT_COLOR;

const CLIENT_COUNT: usize = 2;
const CLIENT_COLORS: [Color; CLIENT_COUNT] = [
    Color::new(1.0, 0.0, 0.0, 1.0),
    Color::new(0.0, 0.0, 1.0, 1.0),
];

pub const PLAYER1_HANDLE: Handle = 1;
pub const PLAYER2_HANDLE: Handle = 2;
const CLIENT_PLAYER_HANDLES: [Handle; CLIENT_COUNT] = [
    PLAYER1_HANDLE,
    PLAYER2_HANDLE
];

const CLIENT_PLAYER_SPAWN_POINTS: [(f32, f32, f32); CLIENT_COUNT] = [
    (750.0, 250.0, 180.0),
    (250.0, 250.0, 0.0)
];


pub struct Server<PROTOCOL: Protocol> {
    clients: [Connection<PROTOCOL>; CLIENT_COUNT],
    handles: HashSet<Handle>,
    // The latest handle to be assigned
    last_handle: Handle,
    // Keys currently held down for each client
    pressed_keys: [HashSet<KeyCode>; CLIENT_COUNT],
    systems: Vec<Box<dyn System>>,
    entities: Vec<Entity>,
    events: VecDeque<Event>,
    delta_time: f32,
}

impl<PROTOCOL: Protocol> Server<PROTOCOL> {
    pub fn main(&mut self) {
        self.await_clients();
        self.spawn_players();
        self.map1();
        self.spawn_border_walls();
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
                self.call_systems();
                if last_broadcast.elapsed() >= MIN_BROADCAST_DURATION {
                    // todo: reason about whether this line should come before or after broadcasting
                    last_broadcast = Instant::now();
                    // todo: new_events and movement_events should come in the right order
                    // this may require some ecs overhauling
                    // actually I'm not sure it's even an issue
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
            last_handle: PLAYER2_HANDLE + 1,
            pressed_keys: [HashSet::new(), HashSet::new()],
            systems: vec![
                Box::new(ControlSystem),
                // Collision system should proceed velocity system
                Box::new(CollisionSystem),
                Box::new(VelocitySystem),
                Box::new(PositionWatcherSystem),
                Box::new(ScaleSystem),
                Box::new(ColorSystem),
                Box::new(HealthSystem),
                Box::new(TtlSystem),
                Box::new(InventorySystem),
                // Should come last, since it handles deleted entities
                Box::new(ReaperSystem),
            ],
            entities: vec![],
            events: VecDeque::new(),
            delta_time: 0.0,
        }
    }

    fn call_systems(&mut self) {
        // todo: maybe don't clone the keys each time
        let mut ctx =
            ServerContext::new(self.pressed_keys.clone(), self.delta_time, self.last_handle);
        for system in &mut self.systems {
            system.update(&mut self.entities, &mut ctx);
        }
        self.despawn_deleted();
        ctx.transfer_state(self);
    }

    fn despawn_deleted(&mut self) {
        let mut despawn_events = VecDeque::new();
        self.entities
            .iter()
            .filter(|entity| entity.deleted())
            .map(|entity| entity.get_handle())
            .map(|handle| Event::Despawn(handle))
            .for_each(|event| despawn_events.push_back(event));
        self.events.append(&mut despawn_events);
        self.entities.retain(|entity| !entity.deleted());
    }

    fn spawn_players(&mut self) {
        for i in 0..self.clients.len() {
            self.spawn_player(i);
        }
    }

    fn spawn_player(&mut self, client_index: usize) {
        let handle = CLIENT_PLAYER_HANDLES[client_index];
        let spawn_point = CLIENT_PLAYER_SPAWN_POINTS[client_index];
        let player = prefabs::player(
            handle,
            client_index,
            spawn_point.0,
            spawn_point.1,
            spawn_point.2,
            CLIENT_COLORS[client_index],
        );
        self.clients[client_index].send(&Event::Yield(handle));
        self.events.push_back(Event::Color(handle, CLIENT_COLORS[client_index]));
        self.spawn(player);
    }

    fn spawn(&mut self, entity: Entity) {
        let handle = entity.get_handle();
        let mesh_type = entity
            .get_component::<MeshType>()
            .cloned()
            .unwrap_or_default();
        self.handles.insert(handle);
        self.events.push_back(Event::Spawn(handle, mesh_type));
        self.entities.push(entity);
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

    fn map1(&mut self) {
        self.spawn(prefabs::heal_item(self.last_handle + 1, 500.0, 250.0));
        self.last_handle += 1;
    }

    fn spawn_border_walls(&mut self) {
        const PURPLE: Color = Color::new(0.7, 0.0, 0.7, 1.0);
        // Upper
        self.spawn(prefabs::wall(self.last_handle + 1, -10.0, -10.0, 1020.0, 20.0, PURPLE));
        self.last_handle += 1;
        // Lower
        self.spawn(prefabs::wall(self.last_handle + 1, -10.0, 490.0, 1020.0, 20.0, PURPLE));
        self.last_handle += 1;
        // Right
        self.spawn(prefabs::wall(self.last_handle + 1, -10.0, 0.0, 20.0, 500.0, PURPLE));
        self.last_handle += 1;
        // Left
        self.spawn(prefabs::wall(self.last_handle + 1, 990.0, 0.0, 20.0, 500.0, PURPLE));
        self.last_handle += 1;
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

enum ServerCommand {
    Spawn(Entity),
    SpawnPlayer(usize),
}

pub struct ServerContext {
    // HashSets of keys pressed on each client
    input_devices: [HashSet<KeyCode>; CLIENT_COUNT],
    delta_time: f32,
    events: VecDeque<Event>,
    commands: VecDeque<ServerCommand>,
    last_handle: Handle,
}

impl ServerContext {
    fn new(
        input_devices: [HashSet<KeyCode>; CLIENT_COUNT],
        delta_time: f32,
        last_handle: Handle,
    ) -> Self {
        Self {
            input_devices,
            delta_time,
            events: VecDeque::new(),
            commands: Default::default(),
            last_handle,
        }
    }

    pub fn spawn(&mut self, mut entity: Entity) {
        let handle = self.last_handle + 1;
        self.last_handle = handle;
        entity = entity.change_handle(handle);
        self.commands.push_back(ServerCommand::Spawn(entity));
    }

    pub fn spawn_player(&mut self, client_index: usize) {
        self.commands.push_back(ServerCommand::SpawnPlayer(client_index));
    }

    fn transfer_state<PROTOCOL: Protocol>(&mut self, server: &mut Server<PROTOCOL>) {
        while !self.commands.is_empty() {
            match self.commands.pop_front() {
                Some(ServerCommand::Spawn(entity)) => server.spawn(entity),
                Some(ServerCommand::SpawnPlayer(client_index)) => server.spawn_player(client_index),
                _ => (),
            }
        }
        server.last_handle = self.last_handle;
        server.events.append(&mut self.events);
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
