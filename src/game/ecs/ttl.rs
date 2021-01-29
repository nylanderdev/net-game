use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;

pub struct TimeToLive(f32);

impl TimeToLive {
    pub fn new(time_to_live: f32) -> Self {
        Self(time_to_live)
    }
}

/// TTL stands for time to live.
pub struct TtlSystem;

impl System for TtlSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            if let Some(time_to_live) = entity.get_component_mut::<TimeToLive>() {
                // It just counts down until we hit zero, then marks an entity for deletion
                // This is useful for getting rid of bullets, so they don't lag up the game
                time_to_live.0 -= ctx.delta_time();
                if time_to_live.0 <= 0.0 {
                    entity.delete();
                }
            }
        }
    }
}
