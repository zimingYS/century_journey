//! 汇总物品基础属性、工具和表现兼容定义。

pub mod presentation;
pub mod tool;

mod category;
mod item;
mod nutrition;

pub use category::ItemCategory;
pub use item::ItemDefinition;
pub use nutrition::{DrinkData, FoodData};
