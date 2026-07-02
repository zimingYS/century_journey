//! # Item Runtime
//!
//! Game 层只保留物品运行时数据：
//! - ItemStack (物品堆叠)
//!
//! Definition/Registry/Loader/Texture 已迁移至 Content 层:
//! - ItemId → shared::item_id
//! - ItemDefinition → content::item::definition
//! - ItemRegistry → content::item::registry
//! - ItemTextureRegistry → content::item::texture
//! - IconDefinition → content::item::texture
//! - ToolData → content::item::definition

pub mod stack;
