//! 聚合库存领域的光标、装备、玩家库存和最近物品状态。

mod cursor;
mod equipment;
mod inventory;
mod recent;

pub use cursor::{CursorData, CursorSource};
pub use equipment::{AccessorySlotDefinitions, EquipmentSlot};
pub use inventory::{InventoryState, LocalInventory, LocalInventoryMut};
pub use recent::RecentItems;
