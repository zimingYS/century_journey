mod plugin;
mod runtime;

pub mod container;
pub mod events;
pub mod interaction;
pub mod item;
pub mod slot;
pub mod state;

pub use plugin::InventoryPlugin;
pub use state::{InventoryState, LocalInventory, LocalInventoryMut};
