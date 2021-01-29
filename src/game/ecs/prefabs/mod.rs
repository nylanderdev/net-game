mod bullet;
pub(crate) mod player;
mod wall;
mod items;

/// A collection of prefab functions which generate entities with common component combinations
/// Examples include player tanks, bullets, healing items and walls

pub use bullet::*;
pub use player::*;
pub use wall::*;
pub use items::*;