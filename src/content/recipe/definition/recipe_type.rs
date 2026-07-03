use serde::{Deserialize, Serialize};

/// 配方类型。
///
/// 这里只定义配方分类，不包含任何业务逻辑。
/// 这边先模范Minecraft的实现方法，以后游戏玩法确定之后进行修改
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeType {
    /// 有序合成
    Shaped,
    /// 无序合成
    Shapeless,
    /// 熔炉烧制
    Furnace,
    /// 高炉
    Blasting,
    /// 熏制
    Smoking,
    /// 营火
    Campfire,
    /// 切石机
    StoneCutting,
    /// 锻造台
    Smithing,
    /// 酿造台
    Brewing,
}
