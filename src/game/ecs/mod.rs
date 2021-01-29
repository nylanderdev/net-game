#![allow(unused)]

mod color;
mod control;
mod health;
mod position;
pub mod prefabs;
mod ttl;
mod velocity;
pub(crate) mod npc;

mod collision;
mod death;
mod scale;
mod item;

pub use item::*;
pub use scale::*;
pub use color::*;
pub use control::*;
pub use health::*;
pub use position::*;
pub use collision::*;
pub use death::*;
pub use ttl::*;
pub use velocity::*;
pub use npc::*;

use crate::game::ServerContext;
use crate::misc::TypeSet;
use crate::net::Handle;
/// This is the entity component system mod. It is largely reused from the last game task
/// Though the components and systems themselves are new, speaking to the extendability of the code ;)
/* This macros provide alternative ways of retrieving components, some allow multiple components at a time,
 * but none of these are currently being used.
 */
macro_rules! entity {
    ($entity:ident as $($C:ty),+) => {
        get_components!($entity| $($C),+)
    };
    ($entity:ident has $($C:ty),+) => {
        has_components!($entity| $($C),+)
    }
}

macro_rules! get_components {
    ($entity:ident| $($C:ty),+) => {
        {
            if has_components!($entity| $($C),+) {
                Some(($($entity.get_component::<$C>().unwrap()),+))
            } else {
                None
            }
        }
    };
}

macro_rules! has_components {
    ($entity:ident| $($C:ty),+) => {
        {
            $($entity.has_component::<$C>())&&+
        }
    };
}

/// An entity is just a fancy wrapper around a TypeSet, but with a handle and a flag for deletion
pub struct Entity {
    components: TypeSet,
    deleted: bool,
    handle: Handle,
}

impl Entity {
    pub fn new(handle: Handle) -> Self {
        Self {
            components: TypeSet::new(),
            deleted: false,
            handle,
        }
    }

    /// Consumes an entity and creates a clone with a different handle
    /// Used by the server to make sure no entities have the same handles,
    /// while still allowing entities to spawn one another (not knowing which handles are free)
    pub fn change_handle(self, new_handle: Handle) -> Self {
        Self {
            components: self.components,
            deleted: self.deleted,
            handle: new_handle,
        }
    }

    pub fn get_handle(&self) -> Handle {
        self.handle
    }

    pub fn put_component<C: Sized + 'static>(&mut self, component: C) {
        self.components.insert::<C>(component);
    }

    pub fn has_component<C: Sized + 'static>(&self) -> bool {
        self.components.contains::<C>()
    }

    pub fn get_component<C: Sized + 'static>(&self) -> Option<&C> {
        self.components.get::<C>()
    }

    pub fn get_component_mut<C: Sized + 'static>(&mut self) -> Option<&mut C> {
        self.components.get_mut::<C>()
    }

    /// Marks an entity for deletion.
    /// This will trigger the ReaperSystem and a DeathsComponent's custom script, if present
    pub fn delete(&mut self) {
        self.deleted = true;
    }

    /// Returns whether the entity is marked for deletion (and should be removed from the game world)
    pub fn deleted(&self) -> bool {
        self.deleted
    }
}

// todo: Systems could probably be function items instead
/// A system is called by the server / game world each frame. It is passed all entities currently
/// in the game world as well as a ServerContext object with API for interacting with the server
/// in certain, limited ways
pub trait System {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext);
}
