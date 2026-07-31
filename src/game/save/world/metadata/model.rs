//! 定义当前世界元数据文档；字段演化由 Serde 默认值负责。

use serde::{Deserialize, Serialize};

use crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION;
use crate::game::world::time::WorldSimulationClock;

/// 世界元数据的当前内存模型。
///
/// 新字段必须提供业务默认值；删除的字段由 MessagePack 命名映射自然忽略。
/// 这里不保存文件格式号，外层文档头才负责区分编码格式。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct LevelData {
    /// 创建或最近保存该文件的游戏版本，来自 Cargo.toml。
    pub game_version: String,
    /// 世界种子。
    pub seed: u64,
    /// 基础地形算法版本；旧世界必须保留原版本，不能随游戏升级自动改变。
    pub generation_version: u32,
    /// 单调递增的服务端模拟 Tick。
    pub simulation_tick: u64,
    /// 绝对游戏分钟，日历字段均由此推导。
    pub game_minute: u64,
    /// 当前游戏分钟内已经经过的固定 Tick。
    pub subminute_tick: u32,
    /// 出生地坐标。
    pub spawn_position: [f32; 3],
    /// 供表现层直接读取的当天小时数，由权威时钟规范化。
    pub time_of_day: f32,
    /// 存档方块编号与稳定标识符的映射。
    pub block_id_map: Vec<(u16, String)>,
}

impl Default for LevelData {
    fn default() -> Self {
        let clock = WorldSimulationClock::default();
        Self {
            game_version: Self::GAME_VERSION.to_string(),
            seed: 0,
            generation_version: CURRENT_GENERATION_VERSION,
            simulation_tick: clock.simulation_tick(),
            game_minute: clock.total_game_minutes(),
            subminute_tick: clock.subminute_tick(),
            spawn_position: [0.0, 70.0, 0.0],
            time_of_day: clock.visual_hour(0.0),
            block_id_map: Vec::new(),
        }
    }
}

impl LevelData {
    /// 构建此存档的游戏包版本。
    pub const GAME_VERSION: &'static str = env!("CARGO_PKG_VERSION");
}
