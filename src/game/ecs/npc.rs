use crate::game::ecs::position::Position;
use crate::game::ecs::velocity::Velocity;
use crate::game::ecs::System;
use crate::game::ecs::{prefabs, ColorComponent, ControlComponent, Entity, Health};
use crate::game::graphics::MeshType;
use crate::game::ServerContext;
use crate::misc::constants::DEFAULT_COLOR;
use crate::net::{Handle, NULL_HANDLE};
use ggez::event::KeyCode;
use ggez::graphics::{Color};
use ggez::nalgebra::{Point2, angle, norm, wrap};
use std::collections::HashSet;
use crate::net::Event::KeyDown;
use rand::Rng;
use nalgebra::Cross;
use ggez::input::mouse::position;
use crate::game::ecs::CollisionComponent;
use crate::game::ecs::DeathComponent;


pub struct NPC{
    //The NPC has a target point that it is trying to reach constantly.
    //The point is updated continually
    target_point: (f32, f32)
}

impl NPC{
    fn new(target_point: (f32, f32)) -> Self{
        Self{
            target_point
        }
    }
    fn set_target_point(&mut self, point: (f32, f32)){
        self.target_point = point
    }
    fn get_target_point(&mut self) -> (f32, f32){
        self.target_point
    }
}

pub fn npc(
    handle: Handle,
    input_device_index: usize,
    x: f32,
    y: f32,
    angle: f32,
    color: Color,
) -> Entity {
    let mut npc = Entity::new(handle);
    npc.put_component(Position::new(x, y, angle));
    npc.put_component(player_control_component(input_device_index));
    npc.put_component(Velocity::new(0.0, 0.0));
    npc.put_component(MeshType::Tank);
    npc.put_component(Health::new(50, 50));
    npc.put_component(ColorComponent::from_color(color));
    npc.put_component(CollisionComponent::new_tank(handle, prefabs::player::player_collision_script));
    npc.put_component(false);
    npc.put_component(NPC::new(generate_random_point()));
    npc
}

//The actions of the tank are carried out by its controlcomponent
fn player_control_component(input_device_index: usize) -> ControlComponent {
    ControlComponent::new(input_device_index, prefabs::player::player_control_script)
}

fn generate_random_point() -> (f32, f32){
    let mut rng = rand::thread_rng();
    let mut x : f32 = rng.gen_range(0.0..1000.0);
    let mut y : f32 = rng.gen_range(0.0..500.0);
    (x, y)
}

fn is_close_enough_to_point(first: (f32, f32), second: (f32, f32)) -> bool{
    (first.0 - second.0).abs() <= 200.0 && (first.1 - second.1).abs() <= 200.0
}

//This modulus works for negative numbers
fn mod_negative(a: f32, n: f32) -> f32{
    (a % n + n) % n
}

//Removes any extra periods of an angle
fn normalize_periodicity(mut angle: f32) -> f32{
    angle = mod_negative(angle, 360.0);
    if angle < 0.0{angle+=360.0}
    angle
}

//Changes the range of angle to (start, end)
fn change_range(mut angle: f32, start: f32, end: f32) -> f32{
    let width = end - start;
    let offset_value = angle - start;
    (offset_value - (offset_value / width).floor()*width) + start
}

//Determine whether the NPC should turn right or left
//Calculated based on the angle of the NPC's velocity and the angle of the desired direction vector
fn right_or_left(first_angle: f32, second_angle: f32) -> i32{
    let angle_diff = (second_angle - first_angle).abs();
    let wrap_around = 360.0 - angle_diff;
    //Is it faster to not wrap around?
    if angle_diff <= wrap_around{
        //If not, is the NPC velocity vector's angle greater than the direction vector's angle?
        if first_angle >= second_angle{
            //Then turn left
            -1
        }
        else{
            //Then turn right
            1
        }
    }
    else{
        if first_angle < second_angle{
            //Turn left to wrap around
            -1
        }
        else{
            //Turn right to wrap around
            1
        }
    }
}


fn update(npc: &mut Entity, ctx: &mut ServerContext, delta_time: f32){
    let mut current_position = Position::new(0.0, 0.0, 0.0);
    if let Some(position) = npc.get_component_mut::<Position>(){
        current_position.set_x(position.get_x());
        current_position.set_y(position.get_y());
        current_position.set_angle(position.get_angle());
    }

    if let Some(npc_2) = npc.get_component_mut::<NPC>(){
        //If it close enough to the target point it will generate a new one
        if is_close_enough_to_point(npc_2.target_point, (current_position.get_x(), current_position.get_y())){
            npc_2.set_target_point(generate_random_point());
        }
        //dir_vec is the vector of our desired
        let dir_vec: (f32, f32) = (npc_2.target_point.0 - current_position.get_x(), npc_2.target_point.1 - current_position.get_y());
        let mut dir_vec_angle = dir_vec.1.atan2(dir_vec.0).to_degrees();
        let mut dir_vec_angle = change_range(dir_vec_angle, 0.0, 360.0);
        let mut current_angle_normalized = normalize_periodicity(current_position.get_angle());

        if right_or_left(current_angle_normalized, dir_vec_angle)== -1{ctx.insert_pressed_key(2, KeyCode::Left)}
        else{ctx.insert_pressed_key(2, KeyCode::Right)}

        //Shoot every 50th update and move forward every 2nd update
        let mut rng = rand::thread_rng();
        let mut x : i32 = rng.gen_range(0..50);
        if x == 1{ctx.insert_pressed_key(2, KeyCode::Space);}
        let mut y : i32 = rng.gen_range(0..2);
        if y == 1{ctx.insert_pressed_key(2, KeyCode::Up);}

    }
}

pub struct NpcSystem;

impl System for NpcSystem{
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(NPC) = entity.get_component_mut::<NPC>(){
                update(entity, ctx, ctx.delta_time());
            }
        }
    }
}