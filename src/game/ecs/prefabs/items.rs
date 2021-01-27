use crate::game::ecs::{Entity, PickUpComponent, CollisionComponent, InventoryComponent, Position, Health, ColorComponent};
use crate::net::Handle;
use crate::game::graphics::MeshType;
use crate::game::ServerContext;

fn pickup_script(me: usize, other: usize, entities: &mut [Entity], ctx: &mut ServerContext) {
    let item = if let Some(item) = entities[me].get_component::<PickUpComponent>() {
        item.get_item()
    } else {
        unreachable!()
    };
    if let Some(inventory) = entities[other].get_component_mut::<InventoryComponent>() {
        if !inventory.has_item() {
            inventory.put_item(item);
            entities[me].delete();
        }
    }
}

fn heal_item_script(user: &mut Entity) {
    if let Some(health) = user.get_component_mut::<Health>() {
        health.set_health((50 - health.get_health()) / 2 + health.get_health());
    }
}

pub fn heal_item(handle: Handle, x: f32, y: f32) -> Entity {
    let mut heal = Entity::new(handle);
    heal.put_component(MeshType::Heal);
    heal.put_component(Position::new(x, y, 0.0));
    heal.put_component(ColorComponent::new(1.0, 0.0, 0.0, 1.0));
    heal.put_component(CollisionComponent::new_item(handle, pickup_script));
    heal.put_component(PickUpComponent::new((MeshType::Heal, heal_item_script)));
    heal
}