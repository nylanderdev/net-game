use crate::misc::{constants::ALL_KEYS, State};
use crate::net::{Connection, Event, EventListener, Handle, Protocol};
use ggez::conf::WindowMode;
use ggez::event::KeyCode;
use ggez::event::{self, EventHandler};
use ggez::graphics::Rect;
use ggez::input::keyboard;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;

pub struct Client<PROTOCOL: Protocol> {
    protocol_marker: PhantomData<PROTOCOL>,
}

impl<PROTOCOL: Protocol> Client<PROTOCOL> {
    pub fn main(&self, remote: Connection<PROTOCOL>) {
        let mut window_mode = WindowMode::default();
        window_mode = window_mode.dimensions(500.0, 500.0);
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
            .window_mode(window_mode)
            .build()
            .expect("aieee, could not create ggez context!");

        // Create an instance of your event handler.
        // Usually, you should provide it with the Context object to
        // use when setting your game up.
        let mut my_game = MyGame::new(&mut ctx, remote);

        // Run!
        match event::run(&mut ctx, &mut event_loop, &mut my_game) {
            Ok(_) => eprintln!("Exited cleanly."),
            Err(e) => eprintln!("Error occured: {}", e),
        }
    }

    pub fn new() -> Self {
        Self {
            protocol_marker: PhantomData::<PROTOCOL>,
        }
    }
}

struct MyGame<PROTOCOL: Protocol> {
    server: Connection<PROTOCOL>,
    started: bool,
    /// Coordinates of objects in the game
    coords: HashMap<Handle, (f32, f32, f32)>,
    /// Handles of objects owned by this client-side game
    key_states: HashMap<KeyCode, State<bool>>,
}

fn new_key_map() -> HashMap<KeyCode, State<bool>> {
    let mut map = HashMap::with_capacity(ALL_KEYS.len());
    for key in &ALL_KEYS {
        map.insert(*key, State::new(false));
    }
    map
}

impl<PROTOCOL: Protocol> MyGame<PROTOCOL> {
    pub fn new(_ctx: &mut Context, remote: Connection<PROTOCOL>) -> Self {
        Self {
            server: remote,
            started: false,
            coords: HashMap::new(),
            key_states: new_key_map(),
        }
    }

    fn await_server(&mut self) {
        while !self.started {
            if let Some(Event::Start) = self.server.recv() {
                self.started = true;
            }
        }
    }

    fn check_keys(&mut self, ctx: &Context) {
        let pressed_keys = keyboard::pressed_keys(&ctx);
        for key in &ALL_KEYS {
            if let Some(key_state) = self.key_states.get_mut(key) {
                if pressed_keys.contains(key) {
                    // key is down, has it just been pressed?
                    if !**key_state {
                        **key_state = true;
                    }
                } else {
                    // key is not down, has it just been released?
                    if **key_state {
                        **key_state = false;
                    }
                }
            }
        }
    }

    fn send_keys(&mut self) {
        for (key, key_state) in &mut self.key_states {
            if key_state.invalidated_since() {
                let key_event = if **key_state {
                    Event::KeyDown(*key)
                } else {
                    Event::KeyUp(*key)
                };
                self.server.send(&key_event);
            }
        }
    }

    fn dispatch_events(&mut self, events: VecDeque<Event>) {
        let mut filtered_events = Vec::new();
        let mut movements = HashMap::new();
        for event in events {
            match event {
                Event::Movement(handle, ..) => {
                    movements.insert(handle, event);
                }
                _ => filtered_events.push(event),
            }
        }
        for event in movements.values() {
            filtered_events.push(event.clone());
        }
        for event in filtered_events {
            self.handle(0, event);
        }
    }
}

impl<PROTOCOL: Protocol> EventHandler for MyGame<PROTOCOL> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.started {
            self.server.send(&Event::Ready);
            self.await_server();
            println!("Game started by server");
        }
        // Check keys pressed or released
        self.check_keys(&ctx);
        // Send info about keys whose state has changed
        self.send_keys();
        let events = self.server.recv_multiple(10000);
        self.dispatch_events(events);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        for coord in self.coords.values() {
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                Rect::new(coord.0 as f32, coord.1 as f32, 50.0, 50.0),
                graphics::BLACK,
            )?;
            let point = ggez::nalgebra::Point2::new(coord.0 + 25.0, coord.1 + 25.0);
            let params = graphics::DrawParam::new()
                .rotation(coord.2.to_radians())
                .offset(point);
            graphics::draw(ctx, &rect, params)?;
        }
        graphics::present(ctx)
    }
}

impl<PROTOCOL: Protocol> EventListener for MyGame<PROTOCOL> {
    fn on_movement(&mut self, _conn_index: usize, handle: Handle, x: f32, y: f32, angle: f32) {
        self.coords.insert(handle, (x, y, angle));
    }

    fn on_spawn(&mut self, _conn_index: usize, handle: Handle) {
        self.coords.insert(handle, (0.0, 0.0, 0.0));
    }
}
