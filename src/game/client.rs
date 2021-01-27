use crate::game::graphics::{generator_from_mesh_type, health_bar, MeshType, inventory_mesh};
use crate::misc::constants::DEFAULT_COLOR;
use crate::misc::{constants::ALL_KEYS, State};
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::KeyCode;
use ggez::event::{self, EventHandler};
use ggez::graphics::{Color, DrawParam, Drawable};
use ggez::input::keyboard;
use ggez::{graphics as gg_graphics, Context, ContextBuilder, GameResult};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use ggez::nalgebra::Vector;

pub struct Client<PROTOCOL: Protocol> {
    protocol_marker: PhantomData<PROTOCOL>,
}

const HALF_WINDOW_WIDTH: f32 = 500.0;
const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 500.0;

impl<PROTOCOL: Protocol> Client<PROTOCOL> {
    pub fn main(&self, remote: Connection<PROTOCOL>) {
        let mut window_mode = WindowMode::default();
        window_mode = window_mode.dimensions(WINDOW_WIDTH, WINDOW_HEIGHT);
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("NetGameTankBattle", "VStenm & RasmusSN")
            .window_mode(window_mode)
            .window_setup(WindowSetup::default().title("Tank Battle (NetGame)"))
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
    meshes: HashMap<Handle, MeshType>,
    dimension: HashMap<Handle, (f32, f32)>,
    /// Handles of objects owned by this client-side game
    key_states: HashMap<KeyCode, State<bool>>,
    player_handle: Handle,
    health: HashMap<Handle, u8>,
    color: HashMap<Handle, Color>,
    inventory: Option<MeshType>,
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
            meshes: HashMap::new(),
            dimension: HashMap::new(),
            key_states: new_key_map(),
            player_handle: NULL_HANDLE,
            health: HashMap::new(),
            color: HashMap::new(),
            inventory: None,
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

    fn render_gui(&self, ctx: &mut Context) -> GameResult<()> {
        let (right_color, right_health, left_color, left_health) = {
            let player_color = self
                .color
                .get(&self.player_handle)
                .cloned()
                .unwrap_or(DEFAULT_COLOR);
            let player_health = self.health.get(&self.player_handle).cloned().unwrap_or(0);
            let player_x = self.coords.get(&self.player_handle).cloned().unwrap_or((0.0, 0.0, 0.0)).0;
            let mut opponent_color = DEFAULT_COLOR;
            let mut opponent_health = 0;
            let mut opponent_x = 0.0;
            for (handle, _) in &self.health {
                if *handle != self.player_handle {
                    opponent_color = self.color.get(&handle).cloned().unwrap_or(DEFAULT_COLOR);
                    opponent_health = self.health.get(&handle).cloned().unwrap_or(0);
                    opponent_x = self.coords.get(&handle).cloned().unwrap_or((0.0, 0.0, 0.0)).0;
                } else {
                    continue;
                }
            }
            if player_x < opponent_x {
                (player_color, player_health, opponent_color, opponent_health)
            } else {
                (opponent_color, opponent_health, player_color, player_health)
            }
        };
        let health_bar_right = health_bar(ctx, 15.0, 15.0, right_color, right_health, 50)?;
        let bar_dim = health_bar_right.dimensions(ctx).unwrap_or_default();
        let health_bar_left = health_bar(
            ctx,
            WINDOW_WIDTH - bar_dim.w - 15.0,
            15.0,
            left_color,
            left_health,
            50,
        )?;
        if self.inventory.is_some() {
            self.render_inventory(ctx)?;
        }
        gg_graphics::draw(ctx, &health_bar_right, DrawParam::default())?;
        gg_graphics::draw(ctx, &health_bar_left, DrawParam::default())
    }

    fn get_dimensions(&self, handle: Handle) -> (f32, f32) {
        self.dimension.get(&handle).cloned().unwrap_or((1.0, 1.0))
    }

    fn render_inventory(&self, ctx: &mut Context) -> GameResult<()>{
        let inventory_bg = inventory_mesh(ctx,
                                          15.0,
                                          WINDOW_HEIGHT - 40.0 - 15.0,
        )?;
        let item_mesh_type = self.inventory.unwrap_or_default();
        let item_mesh = (generator_from_mesh_type(item_mesh_type))(
            ctx, 20.0, WINDOW_HEIGHT - 40.0 - 10.0, Color::new(1.0, 0.0, 0.0, 1.0)
        )?;
        gg_graphics::draw(ctx, &inventory_bg, DrawParam::default())?;
        gg_graphics::draw(ctx, &item_mesh, DrawParam::default())

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
        gg_graphics::clear(ctx, gg_graphics::WHITE);
        for (handle, coord) in &self.coords {
            let color = if let Some(color) = self.color.get(handle) {
                *color
            } else {
                DEFAULT_COLOR
            };
            let scale = self.get_dimensions(*handle);
            let mesh_type = self.meshes.get(handle).cloned().unwrap_or_default();
            let mesh_generator = generator_from_mesh_type(mesh_type);
            let mesh = (mesh_generator)(ctx, coord.0, coord.1, color)?;
            let point = ggez::nalgebra::Point2::new(coord.0, coord.1);
            let params = gg_graphics::DrawParam::new()
                .scale([scale.0, scale.1])
                .rotation(coord.2.to_radians())
                .offset(point);
            gg_graphics::draw(ctx, &mesh, params)?;
        }
        self.render_gui(ctx)?;
        gg_graphics::present(ctx)
    }
}

impl<PROTOCOL: Protocol> EventListener for MyGame<PROTOCOL> {
    fn on_movement(&mut self, _conn_index: usize, handle: Handle, x: f32, y: f32, angle: f32) {
        if self.coords.contains_key(&handle) {
            self.coords.insert(handle, (x, y, angle));
        }
    }

    fn on_yield(&mut self, _conn_index: usize, handle: Handle) {
        self.player_handle = handle;
    }

    fn on_spawn(&mut self, _conn_index: usize, handle: Handle, mesh_type: MeshType) {
        // Place newly spawned object outside of view until a movement event is received
        self.coords.insert(handle, (-100.0, -100.0, 0.0));
        self.meshes.insert(handle, mesh_type);
    }

    fn on_pick_up(&mut self, _conn_index: usize, handle: Handle, mesh_type: MeshType) {
        if handle == self.player_handle {
            self.inventory = match mesh_type {
                MeshType::None => None,
                _ => Some(mesh_type)
            }
        }
    }

    fn on_despawn(&mut self, _conn_index: usize, handle: Handle) {
        self.coords.remove(&handle);
        self.meshes.remove(&handle);
    }

    fn on_health(&mut self, _conn_index: usize, handle: Handle, health: u8) {
        self.health.insert(handle, health);
    }

    fn on_color(&mut self, _conn_index: usize, handle: Handle, color: Color) {
        self.color.insert(handle, color);
    }

    fn on_dimension(&mut self, _conn_index: usize, handle: Handle, width: f32, height: f32) {
        self.dimension.insert(handle, (width, height));
    }
}
