//! 物品的生存属性（食物与饮品）。

use serde::{Deserialize, Serialize};

/// 食物的生存属性。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FoodData {
    /// 恢复的饥饿值。
    pub hunger: f32,
    /// 恢复的饱和度，饱和度会优先承担行动消耗。
    #[serde(default)]
    pub saturation: f32,
}

/// 饮品的生存属性。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DrinkData {
    /// 恢复的口渴值。
    pub thirst: f32,
}
