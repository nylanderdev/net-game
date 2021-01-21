use chrono;
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::conf::WindowMode;
use ggez::event::KeyCode;
use ggez::input::keyboard::KeyCode as VirtualKeyCode;
use ggez::event::{self, EventHandler};
use ggez::graphics::Rect;
use ggez::{graphics, Context, ContextBuilder, GameResult};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use ggez::input::keyboard;
use std::time::{Instant, Duration};
use ggez::timer::check_update_time;
use std::thread::sleep;
use ggez::input::keyboard::KeyMods;

pub struct Client<PROTOCOL: Protocol> {
    protocol_marker: PhantomData<PROTOCOL>,
}

impl<PROTOCOL: Protocol> Client<PROTOCOL> {
    pub fn main(&self, remote: Connection<PROTOCOL>, slow: bool) {
        let mut window_mode = WindowMode::default();
        window_mode = window_mode.dimensions(400.0, 400.0);
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("my_game", "Cool Game Author")
            .window_mode(window_mode)
            .build()
            .expect("aieee, could not create ggez context!");

        // Create an instance of your event handler.
        // Usually, you should provide it with the Context object to
        // use when setting your game up.
        let mut my_game = MyGame::new(&mut ctx, remote, slow);

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
    coords: HashMap<Handle, (i32, i32)>,
    /// Handles of objects owned by this client-side game
    owned: Vec<Handle>,
    code_map: HashMap<KeyCode, (i32, i32)>,
    start: Instant,
    movements: VecDeque<Event>,
    slow: bool,
}

impl<PROTOCOL: Protocol> MyGame<PROTOCOL> {
    pub fn new(_ctx: &mut Context, remote: Connection<PROTOCOL>, slow: bool) -> Self {
        const MOVE_CONST: i32 = 1;
        let mut code_map = HashMap::new();
        code_map.insert(KeyCode::Up, (0, -MOVE_CONST));
        code_map.insert(KeyCode::Down, (0, MOVE_CONST));
        code_map.insert(KeyCode::Left, (-MOVE_CONST, 0));
        code_map.insert(KeyCode::Right, (MOVE_CONST, 0));
        Self {
            server: remote,
            started: false,
            coords: HashMap::new(),
            owned: Vec::new(),
            code_map,
            start: Instant::now(),
            movements: VecDeque::new(),
            slow,
        }
    }

    fn await_server(&mut self) {
        while !self.started {
            if let Some(Event::Start) = self.server.recv() {
                self.started = true;
                self.start = Instant::now();
            }
        }
    }

    fn player_handle(&self) -> Handle {
        // todo: adapt for multiple owned objects
        self.owned.first().cloned().unwrap_or(NULL_HANDLE)
    }

    fn player_coords(&self) -> Option<(i32, i32)> {
        let player_handle = self.player_handle();
        if player_handle != NULL_HANDLE && self.coords.contains_key(&player_handle) {
            Some(self.coords[&self.player_handle()])
        } else {
            None
        }
    }

    fn move_player(&mut self, _ctx: &mut Context) {
        for i in 0..64 {
            if let Some(Event::Movement(handle, x, y)) = self.movements.pop_front() {
                self.coords.insert(handle, (x, y));
            }
        }
        /*
        if let Some(player_coords) = self.player_coords() {
            let _dt = ggez::timer::delta(_ctx);
            let a = keyboard::pressed_keys(_ctx);
            let mut movement = (0, 0);
            for i in a{
                let code = self.code_map.get(i);
                match code {
                    Some(x) => {
                        movement.0 += x.0;
                        movement.1 += x.1;
                    }
                    _ => ()
                }
            }
            self.server.send(&Event::RequestMovement(
                self.player_handle(),
                /*player_coords.0+*/movement.0, /*player_coords.1+*/movement.1
            ));
        }
         */
    }

    fn spam_server(&mut self, ctx: &Context) {
        let _dt = ggez::timer::delta(ctx);
        let a = keyboard::pressed_keys(ctx);
        let mut movement = (0, 0);
        for i in a {
            let code = self.code_map.get(i);
            match code {
                Some(x) => {
                    movement.0 += x.0;
                    movement.1 += x.1;
                }
                _ => ()
            }
        }
        self.server.send(&Event::RequestMovement(
            self.player_handle(),
            /*player_coords.0+*/movement.0, /*player_coords.1+*/movement.1,
        ));
    }
}

impl<PROTOCOL: Protocol> EventHandler for MyGame<PROTOCOL> {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        if !self.started {
            self.server.send(&Event::Ready);
            self.await_server();
        }
        let mut events = self.server.recv_multiple(64);
        while !events.is_empty() {
            self.handle(events.pop_front().unwrap());
        }
        if check_update_time(_ctx, 300) {
            self.move_player(_ctx);
        }
        self.spam_server(&_ctx);
        dbg!(&self.movements.len());
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
            graphics::draw(ctx, &rect, graphics::DrawParam::default());
        }
        graphics::present(ctx)
    }

    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, key_mods: KeyMods, repeat: bool) {
        if let Some(player_coords) = self.player_coords() {
            let _dt = ggez::timer::delta(ctx);
            let a = keyboard::pressed_keys(ctx);
            let mut movement = (0, 0);
            for i in a {
                let code = self.code_map.get(i);
                match code {
                    Some(x) => {
                        movement.0 += x.0;
                        movement.1 += x.1;
                    }
                    _ => ()
                }
            }
            self.server.send(&Event::RequestMovement(
                self.player_handle(),
                /*player_coords.0+*/movement.0, /*player_coords.1+*/movement.1,
            ));
        }
        /*
    dbg!(keycode);
    let movement = match keycode {
        VirtualKeyCode::Left => (-1, 0),
        VirtualKeyCode::Up => (0, -1),
        VirtualKeyCode::Right => (1, 0),
        VirtualKeyCode::Down => (0, 1),
        _ => (0, 0)
    };
    if movement != (0, 0) {
        if self.slow {
            sleep(Duration::from_millis(100));
        }
        self.server.send(&Event::RequestMovement(self.player_handle(),
                                                 movement.0, movement.1
        ));
    }*/
    }
}

impl<PROTOCOL: Protocol> EventListener for MyGame<PROTOCOL> {
    fn on_movement(&mut self, handle: Handle, x: i32, y: i32) {
        self.movements.push_back(Event::Movement(handle, x, y));
        //self.coords.insert(handle, (x, y));
    }

    fn on_yield(&mut self, handle: Handle) {
        self.owned.push(handle);
    }

    fn on_spawn(&mut self, handle: Handle) {
        self.coords.insert(handle, (0, 0));
    }
}
