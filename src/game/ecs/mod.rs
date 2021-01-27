#![allow(unused)]

mod color;
mod control;
mod health;
mod position;
pub mod prefabs;
mod ttl;
mod velocity;
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

use crate::game::ServerContext;
use crate::misc::TypeSet;
use crate::net::Handle;

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

    pub fn delete(&mut self) {
        self.deleted = true;
    }

    pub fn deleted(&self) -> bool {
        self.deleted
    }
}

// todo: Systems could probably be function items instead
pub trait System {
    fn update(&mut self, entities: &mut [Entity], ctx: &mut ServerContext);
}
