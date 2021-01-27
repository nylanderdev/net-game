use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;

pub type DeathScript = fn(&mut Entity, &mut ServerContext);

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
                    (death_component.script)(entity, ctx);
                }
            }
        }
    }
}

