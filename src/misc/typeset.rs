#![allow(dead_code)]

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A struct which allows the storage and retrieval of a single object of each and every type
/// Completely type safe. Simple, but kinda proud of this one.
pub struct TypeSet {
    elements: HashMap<TypeId, Box<dyn Any>>,
}

// The method names are all pretty self-explanatory.
// The code is obvious: use TypeId's as keys in a
// HashMap and guarantee type safety using
// encapsulation
impl TypeSet {
    pub fn new() -> Self {
        Self {
            elements: HashMap::new(),
        }
    }

    pub fn insert<E: Sized + 'static>(&mut self, element: E) {
        self.elements.insert(TypeId::of::<E>(), Box::new(element));
    }

    pub fn contains<E: Sized + 'static>(&self) -> bool {
        self.elements.contains_key(&TypeId::of::<E>())
    }

    pub fn get<E: Sized + 'static>(&self) -> Option<&E> {
        self.elements.get(&TypeId::of::<E>())?.downcast_ref()
    }

    pub fn get_mut<E: Sized + 'static>(&mut self) -> Option<&mut E> {
        self.elements.get_mut(&TypeId::of::<E>())?.downcast_mut()
    }

    pub fn remove<E: Sized + 'static>(&mut self) -> Option<E> {
        *self.elements.remove(&TypeId::of::<E>())?.downcast().ok()?
    }

    pub fn clear(&mut self) {
        self.elements.clear()
    }
}
