pub type Handle = u64;

pub const NULL_HANDLE: Handle = 0;

// NOTE: Events cannot contain Strings yet.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Event {
    Ready,
    Start,
    Movement(Handle, i32, i32),
    RequestMovement(Handle, i32, i32),
    Custom(u32, Vec<u8>),
    Yield(Handle),
    Spawn(Handle),
}

pub trait EventListener {
    fn handle(&mut self, event: Event) {
        match event {
            Event::Ready => self.on_ready(),
            Event::Start => self.on_start(),
            Event::Movement(handle, x, y) => self.on_movement(handle, x, y),
            Event::RequestMovement(handle, x, y) => self.on_request_movement(handle, x, y),
            Event::Custom(kind, data) => self.on_custom(kind, data),
            Event::Yield(handle) => self.on_yield(handle),
            Event::Spawn(handle) => self.on_spawn(handle),
        }
    }

    fn on_ready(&mut self) {}
    fn on_start(&mut self) {}
    fn on_movement(&mut self, _handle: Handle, _x: i32, _y: i32) {}
    fn on_request_movement(&mut self, _handle: Handle, _x: i32, _y: i32) {}
    fn on_custom(&mut self, _kind: u32, _data: Vec<u8>) {}
    fn on_yield(&mut self, _handle: Handle) {}
    fn on_spawn(&mut self, _handle: Handle) {}
}
