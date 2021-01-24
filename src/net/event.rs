use ggez::event::KeyCode;

pub type Handle = u64;

pub const NULL_HANDLE: Handle = 0;

// NOTE: Events cannot contain Strings yet.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Ready,
    Start,
    Movement(Handle, f32, f32, f32),
    RequestMovement(Handle, f32, f32, f32),
    Custom(u32, Vec<u8>),
    Yield(Handle),
    Spawn(Handle),
    KeyDown(KeyCode),
    KeyUp(KeyCode),
}

pub trait EventListener {
    fn handle(&mut self, conn_index: usize, event: Event) {
        match event {
            Event::Ready => self.on_ready(conn_index),
            Event::Start => self.on_start(conn_index),
            Event::Movement(handle, x, y, angle) => {
                self.on_movement(conn_index, handle, x, y, angle)
            }
            Event::RequestMovement(handle, x, y, angle) => {
                self.on_request_movement(conn_index, handle, x, y, angle)
            }
            Event::Custom(kind, data) => self.on_custom(conn_index, kind, data),
            Event::Yield(handle) => self.on_yield(conn_index, handle),
            Event::Spawn(handle) => self.on_spawn(conn_index, handle),
            Event::KeyDown(key_code) => self.on_key_down(conn_index, key_code),
            Event::KeyUp(key_code) => self.on_key_up(conn_index, key_code),
        }
    }

    fn on_ready(&mut self, _conn_index: usize) {}
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
    fn on_spawn(&mut self, _conn_index: usize, _handle: Handle) {}
    fn on_key_up(&mut self, _conn_index: usize, _key_code: KeyCode) {}
    fn on_key_down(&mut self, _conn_index: usize, _key_code: KeyCode) {}
}
