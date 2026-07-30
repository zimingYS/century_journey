//! 汇总各类配方的数据定义。

pub mod ingredient;
pub mod recipe_definition;
pub mod recipe_result;
pub mod recipe_type;
pub mod shaped_recipe;
pub mod shapeless_recipe;

pub use ingredient::*;
pub use recipe_result::*;
pub use recipe_type::*;
