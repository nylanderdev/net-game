use crate::game::ecs::{ColorComponent, Entity, Position, TimeToLive, Velocity, CollisionComponent};
use crate::game::graphics::MeshType;
use crate::net::Handle;
use ggez::graphics::Color;
use crate::game::ServerContext;

fn bullet_collision_script(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext) {
    entities[me].delete();
}

pub fn bullet(handle: Handle, shooter_handle: Handle, x: f32, y: f32, angle: f32, velocity: f32, color: Color) -> Entity {
    let mut bullet = Entity::new(handle);
    bullet.put_component(Position::new(x, y, angle));
    bullet.put_component(Velocity::new(velocity, 0.0));
    bullet.put_component(TimeToLive::new(2.0));
    bullet.put_component(MeshType::Bullet);
    bullet.put_component(ColorComponent::from_color(color));
    bullet.put_component(CollisionComponent::new_bullet(handle, shooter_handle, bullet_collision_script));
    bullet
}
