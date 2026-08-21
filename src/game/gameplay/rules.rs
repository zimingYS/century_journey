//! 定义可存档的通用玩法规则，与具体玩法状态分离。
use bevy::prelude::*;

/// 世界级玩法规则集合。
///
/// 与 WorldSimulationClock 解耦：时钟只管推进，本资源决定推进倍率。
/// 后续 gamerule（keepInventory、randomTickSpeed…）都并入这里。
#[derive(Resource, Debug, Clone, Copy)]
pub struct GameRules {
    /// 游戏时间流逝倍率：1.0 正常，0.5 半速，2.0 双倍速。
    pub time_scale: f32,
}

impl Default for GameRules {
    fn default() -> Self {
        Self { time_scale: 1.0 }
    }
}
