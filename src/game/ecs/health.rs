use crate::game::ecs::{Entity, System};
use crate::game::ServerContext;
use crate::net::Event;

/// A struct which keeps track of health and when it changes
pub struct Health {
    max_health: u8,
    health: u8,
    has_changed: bool,
}

impl Health {
    pub fn new(health: u8, max_health: u8) -> Self {
        Self {
            max_health,
            health,
            has_changed: true,
        }
    }

    pub fn get_health(&self) -> u8 {
        self.health
    }

    pub fn set_health(&mut self, new_health: u8) {
        if new_health <= self.max_health {
            if new_health != self.health {
                self.has_changed = true;
            }
            self.health = new_health;
        }
    }
}

/// A system which relays any changes in health back to the clients
pub struct HealthSystem;

impl System for HealthSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(health) = entity.get_component_mut::<Health>() {
                if health.has_changed {
                    // Tell the clients the new health amount of this entity
                    ctx.push_event(Event::Health(handle, health.health));
                    health.has_changed = false;
                }
            }
        }
    }
}
