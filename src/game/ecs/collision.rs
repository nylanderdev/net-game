use crate::game::ecs::{System, Entity, Velocity, Position};
use crate::game::ServerContext;
use std::collections::HashMap;
use collider::{HbId, Collider, HbProfile, Hitbox, HbEvent};
use crate::net::Handle;
use collider::geom::{Shape, v2};

pub type CollisionScript = fn(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext);

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum CollisionClass {
    Tank(Handle),
    Bullet(Handle),
    Wall(f32, f32),
    Item,
}

impl CollisionClass {
    pub fn is_wall(&self) -> bool {
        match self {
            CollisionClass::Wall(..) => true,
            _ => false
        }
    }
}

#[derive(Copy, Clone)]
pub struct CollisionComponent {
    handle: Handle,
    class: CollisionClass,
    script: CollisionScript,
}

impl CollisionComponent {
    pub fn new_tank(handle: Handle, script: CollisionScript) -> Self {
        Self {
            handle,
            class: CollisionClass::Tank(handle),
            script,
        }
    }
    pub fn new_bullet(handle: Handle, shooter_handle: Handle, script: CollisionScript) -> Self {
        Self {
            handle,
            class: CollisionClass::Bullet(shooter_handle),
            script,
        }
    }

    pub fn new_wall(handle: Handle, script: CollisionScript, width: f32, height: f32) -> Self {
        Self {
            handle,
            class: CollisionClass::Wall(width, height),
            script,
        }
    }

    pub fn new_item(handle: Handle, script: CollisionScript) -> Self {
        Self {
            handle,
            class: CollisionClass::Item,
            script,
        }
    }

    pub fn get_collision_class(&self) -> CollisionClass {
        self.class
    }
}

impl HbProfile for CollisionComponent {
    fn id(&self) -> u64 {
        self.handle
    }

    fn can_interact(&self, other: &Self) -> bool {
        let item_involved =  self.class == CollisionClass::Item || other.class == CollisionClass::Item;
        if item_involved {
            true
        } else if !self.class.is_wall() && !other.class.is_wall() {
            let my_handle = match self.class {
                CollisionClass::Tank(handle) => handle,
                CollisionClass::Bullet(handle) => handle,
                _ => unreachable!()
            };
            let other_handle = match other.class {
                CollisionClass::Tank(handle) => handle,
                CollisionClass::Bullet(handle) => handle,
                _ => unreachable!()
            };
            my_handle != other_handle
        } else {
            // Two walls shouldn't collide
            !(self.class.is_wall() && other.class.is_wall())
        }
    }

    fn cell_width() -> f64 {
        50.0 // idunno
    }

    fn padding() -> f64 {
        0.01
    }
}

pub struct CollisionSystem;

impl System for CollisionSystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        let mut entity_indices_by_handle = HashMap::new();
        for i in 0..entities.len() {
            let handle = entities[i].get_handle();
            if entities[i].has_component::<CollisionComponent>() {
                entity_indices_by_handle.insert(handle, i);
            }
            if let Some(collision_comp) = entities[i].get_component_mut::<CollisionComponent>() {
                // Ensure that handles match, due to certain server side code, they might not
                collision_comp.handle = handle;
            }
        }
        let mut collider = Collider::new();
        for (handle, i) in &entity_indices_by_handle {
            let entity = &entities[*i];
            let collision_comp =
                *entity.get_component::<CollisionComponent>().unwrap();
            let velocity = entity.get_component::<Velocity>();
            let position = entity.get_component::<Position>();
            if position.is_some() {
                let position = position.unwrap();
                let (vel_x, vel_y) = if let Some(velocity) = velocity {
                    let angle = position.get_angle();
                    (
                        angle.to_radians().cos() * velocity.get_velocity(),
                        angle.to_radians().sin() * velocity.get_velocity()
                    )
                } else {
                    (0.0, 0.0)
                };
                let hitbox = hitbox_from_class(
                    collision_comp.class,
                    position.get_x(), position.get_y(),
                    vel_x, vel_y,
                );
                collider.add_hitbox(collision_comp, hitbox);
            }
        }
        let delta_time = ctx.delta_time() as f64;
        while collider.time() < delta_time {
            let time = collider.next_time().min(delta_time);
            collider.set_time(time);
            if let Some((event, profile_1, profile_2)) = collider.next() {
                let entity_index1 = entity_indices_by_handle[&profile_1.handle];
                let entity_index2 = entity_indices_by_handle[&profile_2.handle];
                (profile_1.script)(entity_index1, entity_index2, entities, ctx);
                (profile_2.script)(entity_index2, entity_index1, entities, ctx);
            }
        }
    }
}

fn hitbox_from_class(class: CollisionClass, x: f32, y: f32, vel_x: f32, vel_y: f32) -> Hitbox {
    match class {
        CollisionClass::Tank(_) => Shape::circle(50.0)
            .place(v2(x as f64, y as f64))
            .moving(v2(vel_x as f64, vel_y as f64)),
        CollisionClass::Bullet(_) => Shape::circle(2.5)
            .place(v2(x as f64, y as f64))
            .moving(v2(vel_x as f64, vel_y as f64)),
        CollisionClass::Wall(width, height) => Shape::rect(v2(width as f64, height as f64))
            .place(v2(x as f64 + 0.5 * (width as f64), y as f64 + 0.5 * (height as f64)))
            .moving(v2(vel_x as f64, vel_y as f64)),
        CollisionClass::Item => Shape::circle(25.0)
            .place(v2(x as f64, y as f64))
            .moving(v2(vel_x as f64, vel_y as f64))
    }
}