use crate::game::ecs::position::Position;
use crate::game::ecs::velocity::Velocity;
use crate::game::ecs::{ControlComponent, Entity};
use crate::net::Handle;
use ggez::event::KeyCode;
use std::collections::HashSet;

fn player_control_script(player: &mut Entity, keys: &HashSet<KeyCode>, delta_time: f32) {
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
        velocity.get_velocity()
    } else {
        0.0
    };
    if let Some(position) = player.get_component_mut::<Position>() {
        position.set_angle(position.get_angle() + angle_change);
        if velocity.abs() > VELOCITY_THRESHOLD {
            let angle = position.get_angle();
            let v_x = angle.to_radians().cos() * (velocity);
            let v_y = angle.to_radians().sin() * (velocity);
            position.set_x(position.get_x() + v_x * delta_time);
            position.set_y(position.get_y() + v_y * delta_time);
        }
    }
}

fn player_control_component(input_device_index: usize) -> ControlComponent {
    ControlComponent::new(input_device_index, player_control_script)
}

pub fn player(handle: Handle, input_device_index: usize, x: f32, y: f32, angle: f32) -> Entity {
    let mut entity = Entity::new(handle);
    entity.put_component(Position::new(x, y, angle));
    entity.put_component(Velocity::new(0.0, 0.0));
    entity.put_component(player_control_component(input_device_index));
    entity
}
