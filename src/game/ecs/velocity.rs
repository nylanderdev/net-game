use crate::game::ecs::{Entity, Position, System};
use crate::game::ServerContext;

/// Well, it's just a velocity. I don't think we ever ended up using the angular_velocity though
pub struct Velocity {
    velocity: f32,
    angular_velocity: f32,
}

impl Velocity {
    pub fn new(velocity: f32, angular_velocity: f32) -> Self {
        Self {
            velocity,
            angular_velocity,
        }
    }
    pub fn get_velocity(&self) -> f32 {
        self.velocity
    }
    pub fn get_angular_velocity(&self) -> f32 {
        self.angular_velocity
    }
    pub fn set_velocity(&mut self, velocity: f32) {
        self.velocity = velocity;
    }
    pub fn set_angular_velocity(&mut self, angular_velocity: f32) {
        self.angular_velocity = angular_velocity;
    }
}

/// A physics system which calculates an entities new position from its velocity each frame
pub struct VelocitySystem;

impl System for VelocitySystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        let delta_time = ctx.delta_time();
        for entity in entities {
            if entity.has_component::<Velocity>() {
                let velocity = if let Some(velocity) = entity.get_component::<Velocity>() {
                    velocity.velocity
                } else {
                    unreachable!()
                };
                if let Some(position) = entity.get_component_mut::<Position>() {
                    let angle = position.get_angle();
                    let v_x = angle.to_radians().cos() * (velocity);
                    let v_y = angle.to_radians().sin() * (velocity);
                    position.set_x(position.get_x() + v_x * delta_time);
                    position.set_y(position.get_y() + v_y * delta_time);
                }
            }
        }
    }
}
