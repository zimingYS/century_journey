//! 定义世界级存档元数据及其版本字段。

use serde::{Deserialize, Serialize};

/// 世界数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LevelData {
    /// 整数存档格式版本，用于选择迁移步骤。
    pub version: u32,
    /// 创建或最近保存该文件的游戏版本，来自 Cargo.toml。
    pub game_version: String,
    /// 世界种子
    pub seed: u64,
    /// 基础地形算法版本；旧世界必须保留原版本，不能随游戏升级自动改变。
    pub generation_version: u32,
    /// 单调递增的服务端模拟 Tick。
    pub simulation_tick: u64,
    /// 绝对游戏分钟，日历字段均由此推导。
    pub game_minute: u64,
    /// 当前游戏分钟内已经经过的固定 Tick。
    pub subminute_tick: u32,
    /// 出生地坐标
    pub spawn_position: [f32; 3],
    /// 游戏时间
    pub time_of_day: f32,
    /// 区块方块 ID
    pub block_id_map: Vec<(u16, String)>,
}

impl LevelData {
    /// 当前世界元数据格式版本。
    pub const CURRENT_VERSION: u32 = 3;
    /// 构建此存档的游戏包版本。
    pub const GAME_VERSION: &'static str = env!("CARGO_PKG_VERSION");
}
