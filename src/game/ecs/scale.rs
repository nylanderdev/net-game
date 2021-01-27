use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;
use crate::net::Event;

pub struct Scale {
    width: f32,
    height: f32,
    has_changed: bool,
}

impl Scale {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            has_changed: true,
        }
    }

    pub fn get_width(&self) -> f32 {
        self.width
    }

    pub fn set_width(&mut self, new_width: f32) {
        if new_width != self.width {
            self.has_changed = true;
        }
        self.width = new_width;
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }

    pub fn set_height(&mut self, new_height: f32) {
        if new_height != self.height {
            self.has_changed = true;
        }
        self.height = new_height;
    }
}

impl Default for Scale {
    fn default() -> Self {
        Self {
            width: 1.0,
            height: 1.0,
            has_changed: true
        }
    }
}

pub struct ScaleSystem;

impl System for ScaleSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(scale) = entity.get_component_mut::<Scale>() {
                if scale.has_changed {
                    ctx.push_event(Event::Dimension(handle, scale.width, scale.height));
                    scale.has_changed = false;
                }
            }
        }
    }
}
