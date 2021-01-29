use std::ops::{Deref, DerefMut};

/// A smart pointer which wraps an object and records whether it's mutable state has been accessed
pub struct State<T> {
    t: T,
    invalidated: bool,
}

impl<T> State<T> {
    pub fn new(t: T) -> Self {
        Self {
            t,
            invalidated: false,
        }
    }
    /// Returns a bool signifying whether this state may have changed
    /// since the last call to invalidated_since
    pub fn invalidated(&self) -> bool {
        self.invalidated
    }
    /// Returns a bool signifying whether this state may have changed
    /// since the last call to invalidated_since
    pub fn invalidated_since(&mut self) -> bool {
        // save status before reset
        let old_invalidation_status = self.invalidated();
        // reset status
        self.invalidated = false;
        old_invalidation_status
    }
}

// For any further info on Deref and DerefMut, see the Rust documentation
impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.t
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.invalidated = true;
        &mut self.t
    }
}
