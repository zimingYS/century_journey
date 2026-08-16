//! 定义物品容器的统一访问契约和具体容器实现。

pub mod creative;
pub mod hotbar;
pub mod survival;
pub mod world;

mod kind;
mod layout;
mod traits;

pub use kind::{ContainerKind, ContainerSlotRole};
pub use layout::ContainerLayout;
pub use traits::{GameContainer, InventoryContainer};
