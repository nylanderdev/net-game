use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;
use ggez::event::KeyCode;
use std::collections::HashSet;

pub type ControlScript = fn(&mut Entity, &mut ServerContext, HashSet<KeyCode>, f32);

pub struct ControlComponent {
    // Parameters: owning entity, keys pressed and delta_time
    script: ControlScript,
    // Index of whatever input device the component is subscribed to
    input_device_index: usize,
}

impl ControlComponent {
    pub fn new(input_device_index: usize, script: ControlScript) -> Self {
        Self {
            script,
            input_device_index,
        }
    }

    pub fn get_input_device_index(&self) -> usize {
        self.input_device_index
    }
}

/// A system which gets input/keyboard info from the clients via the server and serves it up
/// to subscribing entities (i.e those with ControlComponents) such as player tanks
pub struct ControlSystem;

impl System for ControlSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            if let Some(component) = entity.get_component::<ControlComponent>() {
                let input_device_index = component.input_device_index;
                // Call the control script, passing in the relevant keyboard information
                (component.script)(
                    entity,
                    ctx,
                    ctx.pressed_keys(input_device_index).clone(),
                    ctx.delta_time(),
                );
            }
        }
    }
}
