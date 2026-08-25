//! 按内容领域组织的定义与交叉引用检查。
//!
//! 各检查器只向上级编译入口暴露一个领域函数，公共路径规则由 paths 模块统一提供。

mod biome;
mod block;
mod cloud;
mod item;
mod locale;
mod loot;
mod ore;
mod recipe;
mod tag;
mod texture;
mod vegetation;

pub(super) use biome::validate_biomes;
pub(super) use block::validate_blocks;
pub(super) use cloud::validate_clouds;
pub(super) use item::{validate_item_name_keys, validate_items};
pub(super) use locale::validate_locales;
pub(super) use loot::validate_loot;
pub(super) use ore::validate_ore_veins;
pub(super) use recipe::validate_recipes;
pub(super) use tag::validate_tags;
pub(super) use texture::validate_textures;
pub(super) use vegetation::validate_tree_species;
