use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;

pub type DeathScript = fn(&mut Entity, &mut ServerContext);

/// A struct holding a script to be executed before the owning entity is removed from the game world
pub struct DeathComponent {
    script: DeathScript
}

impl DeathComponent {
    pub fn new(script: DeathScript) -> Self {
        Self {
            script
        }
    }
}

/// It keeps track of the dying
pub struct ReaperSystem;

impl System for ReaperSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            if entity.deleted() {
                if let Some(death_component) = entity.get_component::<DeathComponent>() {
                    // This is the last frame before the entity is removed from the game world
                    // by the server, so trigger it's death script.
                    (death_component.script)(entity, ctx);
                }
            }
        }
    }
}

