use crate::content::recipe::definition::shaped_recipe::ShapedRecipe;
use crate::content::recipe::definition::shapeless_recipe::ShapelessRecipe;
use serde::{Deserialize, Serialize};

/// 任意一种配方定义。
///
/// 使用内部标签（internally tagged）进行反序列化。
///
/// JSON：
///
/// {
///     "type": "shaped",
///     ...
/// }
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RecipeDefinition {
    Shaped(ShapedRecipe),
    Shapeless(ShapelessRecipe),
}
