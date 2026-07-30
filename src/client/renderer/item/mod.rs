//! 组织物品模型解析、烘焙、缓存以及世界和界面显示适配。

pub mod baked_model;
pub mod baker;
pub mod cache;
pub mod display;
pub mod gui_icon_baker;
pub mod gui_icon_cache;
pub mod mesh_builders;
pub mod renderer;
pub mod resolver;

pub use baked_model::{BakedItemModel, BakedItemModelPart};
pub use baker::{ItemModelBakeContext, ItemModelBaker};
pub use cache::ItemModelCache;
pub use display::ItemDisplayContext;
pub use gui_icon_baker::GuiItemIconBaker;
pub use gui_icon_cache::GuiItemIconCache;
pub use renderer::{ItemRenderContext, ItemRenderer, SpawnedItemEntity};
pub use resolver::{ItemModelResolver, ResolvedItemModel};
