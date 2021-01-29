use crate::game::ecs::{System, Entity};
use crate::game::ServerContext;
use crate::game::graphics::MeshType;
use crate::net::Event;

/// An item is just a mesh used to render it and a script to trigger upon use
pub type Item = (MeshType, ItemUseScript);

pub type ItemUseScript = fn(&mut Entity);

/// A component enabling an entity to be picked up as an item
pub struct PickUpComponent {
    item: Item
}

impl PickUpComponent {
    pub fn new(item: Item) -> Self {
        Self {
            item
        }
    }

    pub fn get_item(&self) -> Item {
        self.item
    }
}

pub struct InventoryComponent {
    item: Option<Item>,
    has_changed: bool,
}

impl InventoryComponent {
    pub fn empty() -> Self {
        Self {
            item: None,
            has_changed: true,
        }
    }

    pub fn put_item(&mut self, item: Item) {
        self.has_changed = true;
        self.item = Some(item);
    }

    pub fn remove_item(&mut self) -> Option<Item> {
        if self.item.is_some() {
            self.has_changed = true;
        }
        let mut removed_item = None;
        std::mem::swap(&mut self.item, &mut removed_item);
        removed_item
    }

    pub fn has_item(&self) -> bool {
        self.item.is_some()
    }
}

/// A system which relays any item pick ups or inventory changes to the clients
pub struct InventorySystem;

impl System for InventorySystem {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext) {
        for entity in entities {
            let handle = entity.get_handle();
            if let Some(inventory) = entity.get_component_mut::<InventoryComponent>() {
                if inventory.has_changed {
                    inventory.has_changed = false;
                    let mesh_type = if inventory.has_item() {
                        inventory.item.unwrap().0
                    } else {
                        MeshType::None
                    };
                    ctx.push_event(Event::PickUp(handle, mesh_type))
                }
            }
        }
    }
}