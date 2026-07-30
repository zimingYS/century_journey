//! 定义具有固定二维排列的合成配方。

use crate::content::recipe::definition::{Ingredient, RecipeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 有序配方
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShapedRecipe {
    /// 配方布局
    pub pattern: Vec<String>,
    /// 字符映射
    pub key: HashMap<char, Ingredient>,
    /// 输出
    pub result: RecipeResult,
}
