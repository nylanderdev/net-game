#![allow(deprecated)]

use crate::game::graphics::MeshType;
use ggez::event::KeyCode;
use ggez::graphics::Color;

pub type Handle = u64;

pub const NULL_HANDLE: Handle = 0;

// NOTE: Events cannot contain Strings yet.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    // Signifies the client is connected properly
    Ready,
    // Tells the client to choose a map or wait for events
    Standby,
    // Tells the client that all map events have been sent and the game will begin
    Start,
    // Tells the client that an entity with handle has a certain x, y, angle
    Movement(Handle, f32, f32, f32),
    #[deprecated]
    // Formerly used by the client to request movement, replaced by KeyUp and KeyDown
    RequestMovement(Handle, f32, f32, f32),
    // Custom events consisting of a type (u32) and data. Used when no fitting event is implemented yet
    // if vec is too long it will break communication
    Custom(u32, Vec<u8>),
    // Used to tell a client which handle is associated with the client's player character
    Yield(Handle),
    // Tells the client to spawn a mesh and associate it with a handle
    Spawn(Handle, MeshType),
    // Tells the client that it has picked up an item and what mesh to render it as
    PickUp(Handle, MeshType),
    // Tells the client to no longer consider (i.e render) an entity/mesh of a certain handle
    Despawn(Handle),
    // Tells the server that a key has just been pressed, client-side
    KeyDown(KeyCode),
    // Tells the server that a key has just been released, client-side
    KeyUp(KeyCode),
    // Tells the client how much health a certain entity has
    Health(Handle, u8),
    // Tells the client what color an entity should be rendered in
    Color(Handle, Color),
    // Tells the client how an entity should be scaled up when rendering
    Dimension(Handle, f32, f32),
    // Tells the client that the game is over for any reason
    GameOver,
    // Used by the client to choose a map
    Map(usize)
}

/// A trait which allows easy routing of events into other methods that want to deal with them
pub trait EventListener {
    /// Submit an event so it can be sent to the proper handler.
    /// Also dispatches a conn_index corresponding to the connection that sent the event
    fn handle(&mut self, conn_index: usize, event: Event) {
        match event {
            Event::Ready => self.on_ready(conn_index),
            Event::Standby => self.on_standby(conn_index),
            Event::Start => self.on_start(conn_index),
            Event::Movement(handle, x, y, angle) => {
                self.on_movement(conn_index, handle, x, y, angle)
            }
            Event::RequestMovement(handle, x, y, angle) => {
                self.on_request_movement(conn_index, handle, x, y, angle)
            }
            Event::Custom(kind, data) => self.on_custom(conn_index, kind, data),
            Event::Yield(handle) => self.on_yield(conn_index, handle),
            Event::Spawn(handle, mesh_type) => self.on_spawn(conn_index, handle, mesh_type),
            Event::PickUp(handle, mesh_type) => self.on_pick_up(conn_index, handle, mesh_type),
            Event::Despawn(handle) => self.on_despawn(conn_index, handle),
            Event::KeyDown(key_code) => self.on_key_down(conn_index, key_code),
            Event::KeyUp(key_code) => self.on_key_up(conn_index, key_code),
            Event::Health(handle, health) => self.on_health(conn_index, handle, health),
            Event::Color(handle, color) => self.on_color(conn_index, handle, color),
            Event::Dimension(handle, width, height) => self.on_dimension(conn_index, handle, width, height),
            Event::GameOver => self.on_game_over(conn_index),
            Event::Map(map_index) => self.on_map(conn_index, map_index)
        }
    }

    /* Handler functions */
    fn on_ready(&mut self, _conn_index: usize) {}
    fn on_standby(&mut self, _conn_index: usize) {}
    fn on_start(&mut self, _conn_index: usize) {}
    fn on_movement(&mut self, _conn_index: usize, _handle: Handle, _x: f32, _y: f32, _angle: f32) {}
    fn on_request_movement(
        &mut self,
        _conn_index: usize,
        _handle: Handle,
        _x: f32,
        _y: f32,
        _angle: f32,
    ) {
    }
    fn on_custom(&mut self, _conn_index: usize, _kind: u32, _data: Vec<u8>) {}
    fn on_yield(&mut self, _conn_index: usize, _handle: Handle) {}
    fn on_spawn(&mut self, _conn_index: usize, _handle: Handle, _mesh_type: MeshType) {}
    fn on_pick_up(&mut self, _conn_index: usize, _handle: Handle, _mesh_type: MeshType) {}
    fn on_despawn(&mut self, _conn_index: usize, _handle: Handle) {}
    fn on_key_up(&mut self, _conn_index: usize, _key_code: KeyCode) {}
    fn on_key_down(&mut self, _conn_index: usize, _key_code: KeyCode) {}
    fn on_health(&mut self, _conn_index: usize, _handle: Handle, _health: u8) {}
    fn on_color(&mut self, _conn_index: usize, _handle: Handle, _color: Color) {}
    fn on_dimension(&mut self, _conn_index: usize, _handle: Handle, _width: f32, _height: f32) {}
    fn on_game_over(&mut self, _conn_index: usize) {}
    fn on_map(&mut self, _conn_index: usize, _map_index: usize) {}
}
