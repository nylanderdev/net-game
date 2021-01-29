use crate::game::ecs::{npc, prefabs, ColorSystem, ControlSystem, Entity, HealthSystem, PositionWatcherSystem, System, TtlSystem, VelocitySystem, CollisionSystem, ReaperSystem, ScaleSystem, InventorySystem, NpcSystem};
use crate::game::graphics::MeshType;
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::event::KeyCode;
use ggez::graphics::{Color, WHITE, BLACK};
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};
use crate::misc::constants::DEFAULT_COLOR;

const CLIENT_COUNT: usize = 2;
const NPC_COUNT: usize = 1;
const CLIENT_COLORS: [Color; CLIENT_COUNT + NPC_COUNT] = [
    Color::new(1.0, 0.0, 0.0, 1.0),
    Color::new(0.0, 0.0, 1.0, 1.0),
    Color::new(0.0, 1.0, 1.0, 1.0),
];

pub const PLAYER1_HANDLE: Handle = 1;
pub const PLAYER2_HANDLE: Handle = 2;
const CLIENT_PLAYER_HANDLES: [Handle; CLIENT_COUNT] = [
    PLAYER1_HANDLE,
    PLAYER2_HANDLE
];

const CLIENT_PLAYER_SPAWN_POINTS: [(f32, f32, f32); CLIENT_COUNT] = [
    (800.0, 250.0, 180.0),
    (200.0, 250.0, 0.0)
];

pub struct Server<PROTOCOL: Protocol> {
    clients: [Connection<PROTOCOL>; CLIENT_COUNT],
    //Every entity in the game has a handle that works as an identifier
    handles: HashSet<Handle>,
    // The latest handle to be assigned
    last_handle: Handle,
    // Keys currently held down for each client
    pressed_keys: [HashSet<KeyCode>; CLIENT_COUNT + NPC_COUNT],
    systems: Vec<Box<dyn System>>,
    //All game objects are considered entities
    entities: Vec<Entity>,
    events: VecDeque<Event>,
    delta_time: f32,
    game_over: bool,
}

impl<PROTOCOL: Protocol> Server<PROTOCOL> {
    pub fn main(&mut self) {
        loop {
            self.purge_state();
            self.await_clients();
            let map_index = self.await_map_choice();
            // Let the clients know the game is ready to start
            self.broadcast_event(&Event::Start);
            self.spawn_npc();
            self.spawn_players();
            match map_index {
                1 => self.map1(), // The bad,
                2 => self.map2(), // The ugly
                _ => ()
            }
            self.spawn_border_walls();
            let mut last_frame = Instant::now();
            let mut last_broadcast = Instant::now();
            const MIN_BROADCAST_DURATION: Duration = Duration::from_micros(0);
            const MIN_FRAME_DURATION: Duration = Duration::from_millis(20);
            while !self.game_over {
                for i in 0..self.clients.len() {
                    if let Some(event) = self.clients[i].recv() {
                        self.handle(i, event);
                    }
                }
                if last_frame.elapsed() >= MIN_FRAME_DURATION {
                    self.delta_time = last_frame.elapsed().as_secs_f32();
                    last_frame = Instant::now();
                    self.call_systems();
                    // Don't spam the clients
                    if last_broadcast.elapsed() >= MIN_BROADCAST_DURATION {
                        last_broadcast = Instant::now();
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
            self.broadcast_event(&Event::GameOver);
        }
    }

    pub fn new(client1: Connection<PROTOCOL>, client2: Connection<PROTOCOL>) -> Self {
        let clients = [client1, client2];
        Self {
            clients,
            handles: Default::default(),
            last_handle: PLAYER2_HANDLE + 1,
            pressed_keys: [HashSet::new(), HashSet::new(), HashSet::new()],
            systems: vec![
                Box::new(NpcSystem),
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
            game_over: false,
        }
    }

    /// Reset a bunch of state between maps
    fn purge_state(&mut self) {
        self.game_over = false;
        self.handles.clear();
        self.pressed_keys.iter_mut().for_each(|s| s.clear());
        self.systems = vec![
            Box::new(NpcSystem),
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
        ];
        self.entities.clear();
        self.events.clear();
    }

    /// Call any ecs Systems part of the game world
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

    /// Send despawn events for all entities that have been marked for deletion and delete the entities
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

    fn spawn_npc(&mut self) {
        let handle = self.last_handle + 1;
        self.last_handle = handle;
        let npc = npc(handle, 2, 500.0, 250.0, 0.0, CLIENT_COLORS[2]);
        self.events
            .push_back(Event::Color(handle, CLIENT_COLORS[2]));
        self.spawn(npc);
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

    /// Spawn an entity (WARNING: Will not reset its handle, so handle must be unique)
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

    // Wait for clients to send a Ready-event before starting game
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
        self.broadcast_event(&Event::Standby);
    }

    fn await_map_choice(&mut self) -> usize {
        // Check whether the guest client is running on this machine
        let hosted_locally = self.clients[1].is_local();
        if !hosted_locally {
            let host = &mut self.clients[0];
            loop {
                match host.recv() {
                    Some(Event::Map(map_index)) => return map_index,
                    _ => ()
                }
            }
        } else {
            // Check both clients
            loop {
                for i in 0..self.clients.len() {
                    match self.clients[i].recv() {
                        Some(Event::Map(map_index)) => return map_index,
                        _ => ()
                    }
                }
            }
        }
    }

    //Sends an event to all clients
    fn broadcast_event(&mut self, event: &Event) {
        for client in &mut self.clients {
            client.send(event);
        }
    }

    // Creates map1
    fn map1(&mut self) {
        const SHIELD_HEIGHT: f32 = 100.0;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 275.0, 125.0));
        self.last_handle += 1;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 275.0, 375.0));
        self.last_handle += 1;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 675.0, 125.0));
        self.last_handle += 1;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 675.0, 375.0));
        self.last_handle += 1;
        self.spawn(prefabs::wall(self.last_handle + 1, 250.0, 250.0 - SHIELD_HEIGHT * 0.5, 10.0, SHIELD_HEIGHT, BLACK));
        self.last_handle += 1;
        self.spawn(prefabs::wall(self.last_handle + 1, 750.0, 250.0 - SHIELD_HEIGHT * 0.5, 10.0, SHIELD_HEIGHT, BLACK));
        self.last_handle += 1;
    }

    // Creates map2
    fn map2(&mut self) {
        const BROWN: Color = Color::new(0.4, 0.3, 0.2, 1.0);
        self.spawn(prefabs::wall(self.last_handle + 1, 400.0, 185.0, 20.0, 150.0, BROWN));
        self.last_handle += 1;
        self.spawn(prefabs::wall(self.last_handle + 1, 605.0, 185.0, 20.0, 150.0, BROWN));
        self.last_handle += 1;
        self.spawn(prefabs::wall(self.last_handle + 1, 500.0, 100.0, 20.0, 100.0, BROWN));
        self.last_handle += 1;
        self.spawn(prefabs::wall(self.last_handle + 1, 500.0, 300.0, 20.0, 100.0, BROWN));
        self.last_handle += 1;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 50.0, 250.0));
        self.last_handle += 1;
        self.spawn(prefabs::heal_item(self.last_handle + 1, 920.0, 250.0));
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

// Keys are the only input we need now. Only need to be updated on changes, and make abstracting away the network easy
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
    GameOver,
}

/// A context object providing an API for some limited interaction with the server made available to ecs Systems
pub struct ServerContext {
    // HashSets of keys pressed on each client
    input_devices: [HashSet<KeyCode>; CLIENT_COUNT + NPC_COUNT],
    delta_time: f32,
    events: VecDeque<Event>,
    commands: VecDeque<ServerCommand>,
    last_handle: Handle,
}

impl ServerContext {
    fn new(
        input_devices: [HashSet<KeyCode>; CLIENT_COUNT + NPC_COUNT],
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

    pub fn pressed_keys(&self, input_device_index: usize) -> &HashSet<KeyCode> {
        &self.input_devices[input_device_index]
    }

    pub fn insert_pressed_key(&mut self, input_device_index: usize, key: KeyCode) {
        self.input_devices[input_device_index].insert(key);
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    pub fn push_event(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn trigger_game_over(&mut self) {
        self.commands.push_back(ServerCommand::GameOver);
    }

    fn transfer_state<PROTOCOL: Protocol>(&mut self, server: &mut Server<PROTOCOL>) {
        while !self.commands.is_empty() {
            match self.commands.pop_front() {
                Some(ServerCommand::Spawn(entity)) => server.spawn(entity),
                Some(ServerCommand::SpawnPlayer(client_index)) => server.spawn_player(client_index),
                Some(ServerCommand::GameOver) => server.game_over = true,
                _ => (),
            }
        }
        server.last_handle = self.last_handle;
        server.events.append(&mut self.events);
    }
}
