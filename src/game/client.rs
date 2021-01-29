use crate::game::graphics::{generator_from_mesh_type, health_bar, MeshType, inventory_mesh};
use crate::misc::constants::DEFAULT_COLOR;
use crate::misc::{constants::ALL_KEYS, State};
use crate::net::{Connection, Event, EventListener, Handle, Protocol, NULL_HANDLE};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{KeyCode, EventsLoop};
use ggez::event::{self, EventHandler};
use ggez::graphics::{Color, DrawParam, Drawable, Text};
use ggez::input::keyboard;
use ggez::{graphics as gg_graphics, Context, ContextBuilder, GameResult};
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use ggez::nalgebra::Vector;
use crate::game::menu::MapMenu;
use crate::game::{PLAYER1_HANDLE, PLAYER2_HANDLE};

pub struct Client<PROTOCOL: Protocol> {
    protocol_marker: PhantomData<PROTOCOL>,
}

const HALF_WINDOW_WIDTH: f32 = 500.0;
const WINDOW_WIDTH: f32 = 1000.0;
const WINDOW_HEIGHT: f32 = 500.0;

impl<PROTOCOL: Protocol> Client<PROTOCOL> {
    pub fn main(&self, mut remote: Connection<PROTOCOL>, host: bool) {
        // We want to make sure the server has successfully found another client before opening any windows
        // We'll let the server know we're standing by.
        remote.send(&Event::Ready);
        await_standby(&mut remote);
        let mut window_mode = WindowMode::default();
        window_mode = window_mode.dimensions(WINDOW_WIDTH, WINDOW_HEIGHT);
        // Make a Context.
        let (mut ctx, mut event_loop) = ContextBuilder::new("NetGameTankBattle", "VStenm & RasmusSN")
            .window_mode(window_mode)
            .window_setup(WindowSetup::default().title("Tank Battle (NetGame)"))
            .build()
            .expect("aieee, could not create ggez context!");

        let mut should_quit = false;
        while !should_quit {
            // Host gets to choose a map
            if host {
                let map_choice = choose_map(&mut ctx, &mut event_loop);
                if map_choice.is_none() {
                    break;
                }
                // Send the choice to the server so things can begin
                remote.send(&Event::Map(map_choice.unwrap()));
            }
            // Create an instance of your event handler.
            // Usually, you should provide it with the Context object to
            // use when setting your game up.
            let mut my_game = MyGame::new(remote);
            // Run!
            match event::run(&mut ctx, &mut event_loop, &mut my_game) {
                Ok(_) if my_game.should_continue => {
                    // we've quit the gameworld but only to select another map
                    ctx.continuing = true;
                }
                // We've actually quit the game
                _ => should_quit = true,
            }
            // Move server back so we can create a new my_game instance
            remote = my_game.server;
            if !should_quit {
                // Wait until server is ready for someone to choose a map
                remote.send(&Event::Ready);
                await_standby(&mut remote);
            }
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
    /// Keys and whether they're held down or not. Wrapped in a State to record change
    key_states: HashMap<KeyCode, State<bool>>,
    player_handle: Handle,
    health: HashMap<Handle, u8>,
    color: HashMap<Handle, Color>,
    inventory: Option<MeshType>,
    starting_events: VecDeque<Event>,
    should_continue: bool,
    game_over: bool,
}

fn new_key_map() -> HashMap<KeyCode, State<bool>> {
    // Just creates a new HashMap with a false (not held down) entry for all keys
    // This can't be a hashset due to the fact that we're recording change with the State wrapper
    let mut map = HashMap::with_capacity(ALL_KEYS.len());
    for key in &ALL_KEYS {
        map.insert(*key, State::new(false));
    }
    map
}

impl<PROTOCOL: Protocol> MyGame<PROTOCOL> {
    pub fn new(remote: Connection<PROTOCOL>) -> Self {
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
            starting_events: VecDeque::new(),
            should_continue: false,
            game_over: false,
        }
    }

    fn await_start(&mut self) {
        if !self.started {
            match self.server.recv() {
                Some(Event::Start) => self.started = true,
                Some(Event::GameOver) => {
                    self.game_over = true;
                    self.started = true;
                }
                Some(event) => self.starting_events.push_back(event),
                _ => (),
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

    /// Handle incoming events
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
            // Check for other health values to get one to render
            for (handle, _) in &self.health {
                // Find the one that isn't our players, but that belongs to a player (i.e not an NPC)
                if *handle != self.player_handle && (*handle == PLAYER1_HANDLE || *handle == PLAYER2_HANDLE) {
                    opponent_color = self.color.get(&handle).cloned().unwrap_or(DEFAULT_COLOR);
                    opponent_health = self.health.get(&handle).cloned().unwrap_or(0);
                    opponent_x = self.coords.get(&handle).cloned().unwrap_or((0.0, 0.0, 0.0)).0;
                } else {
                    continue;
                }
            }
            // Place the rightmost player's health to the right and vice versa
            if player_x < opponent_x {
                (player_color, player_health, opponent_color, opponent_health)
            } else {
                (opponent_color, opponent_health, player_color, player_health)
            }
        };
        // Render some health bars
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
        // If there's any inventory item for my player, render that too
        if self.inventory.is_some() {
            self.render_inventory(ctx)?;
        }
        gg_graphics::draw(ctx, &health_bar_right, DrawParam::default())?;
        gg_graphics::draw(ctx, &health_bar_left, DrawParam::default())
    }

    fn get_dimensions(&self, handle: Handle) -> (f32, f32) {
        self.dimension.get(&handle).cloned().unwrap_or((1.0, 1.0))
    }

    fn render_inventory(&self, ctx: &mut Context) -> GameResult<()> {
        // Make a translucent background for our inventory
        let inventory_bg = inventory_mesh(ctx,
                                          15.0,
                                          WINDOW_HEIGHT - 40.0 - 15.0,
        )?;
        let item_mesh_type = self.inventory.unwrap_or_default();
        // Get the mesh of the inventory item to render
        let item_mesh = (generator_from_mesh_type(item_mesh_type))(
            ctx, 20.0, WINDOW_HEIGHT - 40.0 - 10.0, Color::new(1.0, 0.0, 0.0, 1.0),
        )?;
        gg_graphics::draw(ctx, &inventory_bg, DrawParam::default())?;
        gg_graphics::draw(ctx, &item_mesh, DrawParam::default())
    }
}

impl<PROTOCOL: Protocol> EventHandler for MyGame<PROTOCOL> {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.started {
            self.await_start();
        } else if self.game_over {
            event::quit(ctx);
        } else {
            if !self.starting_events.is_empty() {
                let mut moved_events = VecDeque::with_capacity(0);
                std::mem::swap(&mut self.starting_events, &mut moved_events);
                self.dispatch_events(moved_events);
            }
            // Check keys pressed or released
            self.check_keys(&ctx);
            // Send info about keys whose state has changed
            self.send_keys();
            let events = self.server.recv_multiple(10000);
            self.dispatch_events(events);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        if !self.started {
            gg_graphics::clear(ctx, gg_graphics::BLACK);
            let text = Text::new("A map is being selected or loaded. Please and thank you.");
            gg_graphics::draw(ctx, &text, DrawParam::default())?;
        } else {
            gg_graphics::clear(ctx, gg_graphics::WHITE);
            for (handle, coord) in &self.coords {
                // Just access a bunch of properties of our game objects and render them using them
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
        }
        gg_graphics::present(ctx)
    }
}

impl<PROTOCOL: Protocol> EventListener for MyGame<PROTOCOL> {
    fn on_movement(&mut self, _conn_index: usize, handle: Handle, x: f32, y: f32, angle: f32) {
        // Only insert coords for spawned objects, lest you might respawn something recently despawned
        if self.coords.contains_key(&handle) {
            self.coords.insert(handle, (x, y, angle));
        }
    }

    fn on_yield(&mut self, _conn_index: usize, handle: Handle) {
        // The server told me this is my player_handle
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

    fn on_game_over(&mut self, _conn_index: usize) {
        self.should_continue = true;
        self.game_over = true;
    }
}

fn await_standby<PROTOCOL: Protocol>(conn: &mut Connection<PROTOCOL>) {
    loop {
        match conn.recv() {
            Some(Event::Standby) => break,
            _ => ()
        }
    }
}

fn choose_map(ctx: &mut Context, event_loop: &mut EventsLoop) -> Option<usize> {
    let mut map_menu = MapMenu::new();
    event::run(ctx, event_loop, &mut map_menu);
    if map_menu.choice().is_some() {
        // We only want to continue if they've actually selected a map,
        // otherwise we'll close the game
        ctx.continuing = true;
    }
    map_menu.choice()
}