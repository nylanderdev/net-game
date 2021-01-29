use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;
use crate::net::Event;
use ggez::graphics::Color;

/// It's a wrapper for a color that keeps track of when its color changes
pub struct ColorComponent {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
    has_changed: bool,
}

impl ColorComponent {
    pub fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
            has_changed: true,
        }
    }
    pub fn from_color(color: Color) -> Self {
        Self::new(color.r, color.g, color.b, color.a)
    }
    pub fn get_color(&self) -> Color {
        Color::new(self.red, self.green, self.blue, self.alpha)
    }
    pub fn get_red(&self) -> f32 {
        self.red
    }
    pub fn get_green(&self) -> f32 {
        self.green
    }
    pub fn get_blue(&self) -> f32 {
        self.blue
    }
    pub fn get_alpha(&self) -> f32 {
        self.alpha
    }
    pub fn get_rgba(&self) -> (f32, f32, f32, f32) {
        (self.red, self.green, self.blue, self.blue)
    }
    pub fn set_red(&mut self, red: f32)  {
        if red != self.red {
            self.has_changed = true;
        }
        self.red = red;
    }
    pub fn set_green(&mut self, green: f32)  {
        if green != self.green {
            self.has_changed = true;
        }
        self.green = green;
    }
    pub fn set_blue(&mut self, blue: f32)  {
        if blue != self.blue {
            self.has_changed = true;
        }
        self.blue = blue;
    }
    pub fn set_alpha(&mut self, alpha: f32)  {
        if alpha != self.alpha {
            self.has_changed = true;
        }
        self.alpha = alpha;
    }
}

/// A system which checks if any colored entity changes color and relays that change to the clients
pub struct ColorSystem;

impl System for ColorSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(color) = entity.get_component_mut::<ColorComponent>() {
                if color.has_changed {
                    ctx.push_event(Event::Color(handle, color.get_color()));
                    color.has_changed = false;
                }
            }
        }
    }
}
