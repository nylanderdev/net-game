use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;
use crate::net::Event;

/// A struct containing coordinates and info about whether they've changed since last
pub struct Position {
    // It goes x, y, angle
    coords_and_angle: (f32, f32, f32),
    has_changed: bool,
}

impl Position {
    pub fn new(x: f32, y: f32, angle: f32) -> Self {
        Self {
            coords_and_angle: (x, y, angle),
            has_changed: true,
        }
    }
    pub fn get_x(&self) -> f32 {
        self.coords_and_angle.0
    }
    pub fn get_y(&self) -> f32 {
        self.coords_and_angle.1
    }
    pub fn get_angle(&self) -> f32 {
        self.coords_and_angle.2
    }
    pub fn set_x(&mut self, x: f32) {
        if !self.has_changed && self.coords_and_angle.0 != x {
            self.has_changed = true;
        }
        self.coords_and_angle.0 = x;
    }
    pub fn set_y(&mut self, y: f32) {
        if !self.has_changed && self.coords_and_angle.1 != y {
            self.has_changed = true;
        }
        self.coords_and_angle.1 = y;
    }
    pub fn set_angle(&mut self, angle: f32) {
        if !self.has_changed && self.coords_and_angle.2 != angle {
            self.has_changed = true;
        }
        self.coords_and_angle.2 = angle;
    }
    /// Returns whether this position may have changed since last time this function was called
    pub fn has_changed_since_last(&mut self) -> bool {
        let had_changed = self.has_changed;
        self.has_changed = false;
        had_changed
    }
}

/// A system which relays any change in position to the clients
pub struct PositionWatcherSystem;

impl System for PositionWatcherSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(position) = entity.get_component_mut::<Position>() {
                if position.has_changed_since_last() {
                    ctx.push_event(Event::Movement(
                        handle,
                        position.get_x(),
                        position.get_y(),
                        position.get_angle(),
                    ));
                }
            }
        }
    }
}
