use crate::game::ecs::{Entity, Position, CollisionComponent, Scale, ColorComponent};
use crate::net::Handle;
use crate::game::ServerContext;
use crate::game::graphics::MeshType;
use ggez::graphics::Color;

fn wall_collision_script(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext) {}

pub fn wall(handle: Handle, x: f32, y: f32, w: f32, h: f32, color: Color) -> Entity {
    let mut wall = Entity::new(handle);
    wall.put_component(Position::new(x, y, 0.0));
    wall.put_component(Scale::new(w, h));
    wall.put_component(ColorComponent::from_color(color));
    wall.put_component(CollisionComponent::new_wall(handle, wall_collision_script, w, h));
    wall.put_component(MeshType::Wall);
    wall
}