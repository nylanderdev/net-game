use crate::game::ecs::position::Position;
use crate::game::ecs::velocity::Velocity;
use crate::game::ecs::{prefabs, ColorComponent, ControlComponent, Entity, Health, DeathComponent, CollisionClass, InventoryComponent};
use crate::game::graphics::MeshType;
use crate::game::ServerContext;
use crate::misc::constants::DEFAULT_COLOR;
use crate::net::{Handle, NULL_HANDLE};
use ggez::event::KeyCode;
use ggez::graphics::Color;
use std::collections::HashSet;
use crate::game::ecs::collision::CollisionComponent;

fn player_death_script(me: &mut Entity, ctx: &mut ServerContext) {
    let client_index = if let Some(control_comp) = me.get_component::<ControlComponent>() {
        control_comp.get_input_device_index()
    } else {
        unreachable!()
    };
    println!("Player {} was killed!", client_index + 1);
    ctx.spawn_player(client_index);
}

fn player_collision_script(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext) {
    let collision_class = entities[other].get_component::<CollisionComponent>()
        .expect("Fatal error in player_collision_script").get_collision_class();
    match collision_class {
        CollisionClass::Bullet(_) => if let Some(health) = entities[me].get_component_mut::<Health>() {
            if health.get_health() > 0 {
                let new_health = health.get_health() - 1;
                health.set_health(new_health);
                if new_health == 0 {
                    entities[me].delete();
                }
            }
        }
        CollisionClass::Tank(..) => {
            if let Some(velocity) = entities[me].get_component_mut::<Velocity>() {
                velocity.set_velocity(velocity.get_velocity() * 0.5);
            }
        }
        CollisionClass::Wall(..) => {
            // Todo: add better stopping (i.e more selective in regards to vertical/horizontal velocity)
            if let Some(velocity) = entities[me].get_component_mut::<Velocity>() {
                velocity.set_velocity(0.0);
            }
        }
        _ => ()
    }
}

fn player_control_script(
    player: &mut Entity,
    ctx: &mut ServerContext,
    keys: HashSet<KeyCode>,
    delta_time: f32,
) {
    const VELOCITY_THRESHOLD: f32 = 0.01;
    let (mut speed_change, mut angle_change): (f32, f32) = (0.0, 0.0);
    if keys.contains(&KeyCode::Up) {
        speed_change += 1000.0 * delta_time
    }
    if keys.contains(&KeyCode::Down) {
        speed_change -= 1000.0 * delta_time
    }
    if keys.contains(&KeyCode::Right) {
        angle_change += 180.0 * delta_time
    }
    if keys.contains(&KeyCode::Left) {
        angle_change -= 180.0 * delta_time
    }
    // The percentage of speed to decrease per second
    const FRICTION_CONSTANT: f32 = 0.90;
    let friction = (1.0 - FRICTION_CONSTANT).powf(delta_time);
    let velocity = if let Some(velocity) = player.get_component_mut::<Velocity>() {
        velocity.set_velocity(
            // speed_change and friction
            (velocity.get_velocity() + speed_change) * friction,
        );
        if velocity.get_velocity().abs() <= VELOCITY_THRESHOLD {
            velocity.set_velocity(0.0);
        }
        velocity.get_velocity()
    } else {
        0.0
    };
    let (x, y, angle) = if let Some(position) = player.get_component_mut::<Position>() {
        position.set_angle(position.get_angle() + angle_change);
        (position.get_x(), position.get_y(), position.get_angle())
    } else {
        unreachable!();
    };
    let color = if let Some(color) = player.get_component::<ColorComponent>() {
        color.get_color()
    } else {
        DEFAULT_COLOR
    };
    if let Some(firing) = player.get_component_mut::<bool>() {
        if keys.contains(&KeyCode::Space) {
            if !*firing {
                *firing = true;
                ctx.spawn(prefabs::bullet(NULL_HANDLE, player.get_handle(), x, y, angle, 1000.0, color));
            }
        } else {
            *firing = false;
        }
    }
    if keys.contains(&KeyCode::Key1) {
        if let Some(inventory) = player.get_component_mut::<InventoryComponent>() {
            if inventory.has_item() {
                let item = inventory.remove_item().unwrap();
                (item.1)(player);
            }
        }
    }
}

fn player_control_component(input_device_index: usize) -> ControlComponent {
    ControlComponent::new(input_device_index, player_control_script)
}

pub fn player(
    handle: Handle,
    input_device_index: usize,
    x: f32,
    y: f32,
    angle: f32,
    color: Color,
) -> Entity {
    let mut player = Entity::new(handle);
    player.put_component(Position::new(x, y, angle));
    player.put_component(Velocity::new(0.0, 0.0));
    player.put_component(player_control_component(input_device_index));
    player.put_component(MeshType::Tank);
    player.put_component(Health::new(50, 50));
    player.put_component(ColorComponent::from_color(color));
    player.put_component(CollisionComponent::new_tank(handle, player_collision_script));
    player.put_component(DeathComponent::new(player_death_script));
    player.put_component(InventoryComponent::empty());
    // firing status
    player.put_component(false);
    player
}
