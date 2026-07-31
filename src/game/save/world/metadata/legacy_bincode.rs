//! 冻结世界元数据历史 bincode 布局，仅用于读取已有存档。
//!
//! 这些结构按布局能力命名且永不扩展。新的字段变化由当前 MessagePack
//! 文档的命名字段兼容处理，不再为每次字段变化增加版本结构体。

use serde::{Deserialize, Serialize};

/// 没有 magic、使用浮点格式号和浮点时间的最早布局。
#[derive(Serialize, Deserialize)]
pub(super) struct FloatVersionLevel {
    pub(super) seed: u64,
    pub(super) spawn_position: [f32; 3],
    pub(super) time_of_day: f32,
    pub(super) block_id_map: Vec<(u16, String)>,
    pub(super) version: f32,
}

/// 首次记录游戏版本，但尚未记录地形算法版本的布局。
#[derive(Serialize, Deserialize)]
pub(super) struct GameVersionLevel {
    pub(super) version: u32,
    pub(super) game_version: String,
    pub(super) seed: u64,
    pub(super) spawn_position: [f32; 3],
    pub(super) time_of_day: f32,
    pub(super) block_id_map: Vec<(u16, String)>,
}

/// 已记录地形算法版本，但仍使用浮点时间的布局。
#[derive(Serialize, Deserialize)]
pub(super) struct GenerationLevel {
    pub(super) version: u32,
    pub(super) game_version: String,
    pub(super) seed: u64,
    pub(super) generation_version: u32,
    pub(super) spawn_position: [f32; 3],
    pub(super) time_of_day: f32,
    pub(super) block_id_map: Vec<(u16, String)>,
}

/// 最后一个按字段顺序编码、已经带有权威时钟的布局。
#[derive(Serialize, Deserialize)]
pub(super) struct SimulationClockLevel {
    pub(super) version: u32,
    pub(super) game_version: String,
    pub(super) seed: u64,
    pub(super) generation_version: u32,
    pub(super) simulation_tick: u64,
    pub(super) game_minute: u64,
    pub(super) subminute_tick: u32,
    pub(super) spawn_position: [f32; 3],
    pub(super) time_of_day: f32,
    pub(super) block_id_map: Vec<(u16, String)>,
}
