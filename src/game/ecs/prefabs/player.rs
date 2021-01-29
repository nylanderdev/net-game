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

pub fn player_death_script(me: &mut Entity, ctx: &mut ServerContext) {
    // Check which client is controlling the player so we can hook up a respawned player to it
    let client_index = if let Some(control_comp) = me.get_component::<ControlComponent>() {
        control_comp.get_input_device_index()
    } else {
        unreachable!()
    };
    // Respawn the player
    ctx.spawn_player(client_index);
    // Trigger a game over for now. Todo: Make sure it takes a couple of deaths until a game over
    ctx.trigger_game_over();
}

pub fn player_collision_script(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext) {
    // This is a collision script, so if it the entities don't have CollisionComponents,
    // something mighty weird must be happening
    let collision_class = entities[other].get_component::<CollisionComponent>()
        .expect("Fatal error in player_collision_script").get_collision_class();
    // What have we collided with?
    match collision_class {
        // A bullet?! Guess I'll die (in 49 more shots)
        CollisionClass::Bullet(_) => if let Some(health) = entities[me].get_component_mut::<Health>() {
            // This check is important since there might multiple bullets damaging a player in a single frame
            // and since health is an unsigned integer we don't want to underflow it and crash the game!
            if health.get_health() > 0 {
                let new_health = health.get_health() - 1;
                health.set_health(new_health);
                if new_health == 0 {
                    // o o f - death. One could argue this should be handled by the HealthSystem.
                    // Buuut it works and you might wanna customize future health events
                    // and I don't feel like making any more script aliases
                    entities[me].delete();
                }
            }
        }
        CollisionClass::Tank(..) => {
            if let Some(velocity) = entities[me].get_component_mut::<Velocity>() {
                // Slow down while driving over another tank.
                // if we slow down to a halt we might accidentally get stuck in a respawning tank,
                // so just half off
                velocity.set_velocity(velocity.get_velocity() * 0.5);
            }
        }
        CollisionClass::Wall(..) => {
            // Stop things from going through walls.
            // This will make walls feel kinda sticky since you will stop fully and not be able
            // to glide against them, but it works. Could be fixed by checking whether the player
            // is aimed straight at the wall or not, then doing some vector math / trig
            // Todo: add better stopping (i.e more selective in regards to vertical/horizontal velocity)
            if let Some(velocity) = entities[me].get_component_mut::<Velocity>() {
                velocity.set_velocity(0.0);
            }
        }
        _ => ()
    }
}

/// This enables a client to control a player tank
pub fn player_control_script(
    player: &mut Entity,
    ctx: &mut ServerContext,
    // The keys currently held down on the client's keyboard
    keys: HashSet<KeyCode>,
    delta_time: f32,
) {
    const VELOCITY_THRESHOLD: f32 = 0.01;
    let (mut speed_change, mut angle_change): (f32, f32) = (0.0, 0.0);
    // Accelerate forwards (1000 px/s, or one 3600 kilopixels per hour, wew!)
    if keys.contains(&KeyCode::Up) {
        speed_change += 1000.0 * delta_time
    }
    // Accelerate backwards
    if keys.contains(&KeyCode::Down) {
        speed_change -= 1000.0 * delta_time
    }
    // Turn at an angular velocity of 180 degrees per second
    if keys.contains(&KeyCode::Right) {
        angle_change += 180.0 * delta_time
    }
    if keys.contains(&KeyCode::Left) {
        angle_change -= 180.0 * delta_time
    }
    // The percentage of speed to decrease per second.
    // This results in a terminal velocity, so we don't have to worry about accumulating too much
    // from the acceleration. Real clever
    const FRICTION_CONSTANT: f32 = 0.90;
    // We're multiplying here, so the delta_time will be accounted for using delta_time
    // The algebra checks out ...I think.
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
        // If the player doesn't have a position, that's serious cause for concern.
        // Fortunately, it won't happen. All players are spawned with positions ready.
        unreachable!();
    };
    // We need to check the color of the player to color their bullets the same color
    // If we don't, it'll be ugly and people will notice how bullets spawn on top of tanks.
    let color = if let Some(color) = player.get_component::<ColorComponent>() {
        color.get_color()
    } else {
        DEFAULT_COLOR
    };
    // The bool component in the entity says whether the tank recently fired. We don't want any bullet spam
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
    // If the player presses the key I, that means they want to use whatever's in their inventory
    if keys.contains(&KeyCode::I) {
        if let Some(inventory) = player.get_component_mut::<InventoryComponent>() {
            if inventory.has_item() {
                let item = inventory.remove_item().unwrap();
                // Trigger the item's use script
                (item.1)(player);
            }
        }
    }
}

// It's just more concise this way
fn player_control_component(input_device_index: usize) -> ControlComponent {
    ControlComponent::new(input_device_index, player_control_script)
}

/// A player is complicated, so it's got lots of components, but don't fret.
pub fn player(
    handle: Handle,
    // This number indicates which client controls the player
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
