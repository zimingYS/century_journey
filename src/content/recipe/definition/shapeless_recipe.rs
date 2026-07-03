use crate::content::recipe::definition::{Ingredient, RecipeResult};
use serde::{Deserialize, Serialize};

/// 无序合成配方。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapelessRecipe {
    /// 输入材料
    pub ingredients: Vec<Ingredient>,
    /// 输出
    pub result: RecipeResult,
}
