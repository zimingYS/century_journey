//! # Item
//!
//! 物品定义。
//!
//! 定义物品属性、资源及注册信息。

pub mod definition;
pub mod loader;
pub mod model;
pub mod plugin;
pub mod registry;
pub mod texture;

// Re-export commonly used types
pub use definition::ItemCategory;
pub use definition::ItemDefinition;
pub use definition::tool::{ToolData, ToolTier, ToolType};
pub use registry::registry::ItemRegistry;
pub use texture::icon::IconDefinition;
pub use texture::registry::ItemTextureRegistry;
pub use texture::registry::load_item_textures_system;
