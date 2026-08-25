//! 调试浮层的纯文本组织逻辑：罗盘方位判定、季节标签与行文本组装。
//!
//! 本模块不读取 ECS 资源，数据由调用方采集后传入，便于单元测试。
//! 方位与季节返回本地化键，由 [`build_lines`] 查表得到最终译文。

use crate::engine::localization::Localization;
use crate::game::world::time::Season;
use bevy::math::{IVec3, Vec3};

/// 依据前向向量判定八向罗盘方位，返回 `debug.direction.*` 本地化键。
///
/// 方向约定与主流体素游戏一致：北为 -Z、东为 +X，每 45 度一个扇区。
pub fn compass_direction(forward: Vec3) -> &'static str {
    // 北 = -Z（角度 0），东 = +X（角度 90 度），角度顺时针展开。
    let angle = forward.x.atan2(-forward.z).to_degrees().rem_euclid(360.0) as i32;
    match angle {
        0..=22 | 337.. => "debug.direction.north",
        23..=67 => "debug.direction.north-east",
        68..=112 => "debug.direction.east",
        113..=157 => "debug.direction.south-east",
        158..=202 => "debug.direction.south",
        203..=247 => "debug.direction.south-west",
        248..=292 => "debug.direction.west",
        _ => "debug.direction.north-west",
    }
}

/// 返回季节的本地化键（`debug.season.*`）。
pub fn season_label(season: Season) -> &'static str {
    match season {
        Season::Spring => "debug.season.spring",
        Season::Summer => "debug.season.summer",
        Season::Autumn => "debug.season.autumn",
        Season::Winter => "debug.season.winter",
    }
}

/// 玩家朝向信息：罗盘方位与两个欧拉角（角度制）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FacingInfo {
    /// 八向罗盘方位的本地化键。
    pub direction: &'static str,
    /// 偏航角（绕 Y 轴，度）。
    pub yaw_deg: f32,
    /// 俯仰角（绕 X 轴，度）。
    pub pitch_deg: f32,
}

/// 玩家所在区块坐标与区块内局部坐标（0..16）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkInfo {
    /// 区块三维坐标（可为负）。
    pub chunk: IVec3,
    /// 区块内的局部方块坐标。
    pub local: IVec3,
}

/// 世界日历与时间倍率信息。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClockInfo {
    /// 自世界创建起从一开始计数的游戏日。
    pub game_day: u64,
    /// 当前小时（0..24）。
    pub hour: u8,
    /// 当前分钟（0..60）。
    pub minute: u8,
    /// 季节本地化键。
    pub season: &'static str,
    /// 当前时间流速倍率。
    pub time_scale: f32,
}

/// 区块流送计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCounts {
    /// 世界状态中已加载的区块数。
    pub loaded: usize,
    /// 流送配置预期的区块数。
    pub expected: usize,
    /// 已进入渲染态的区块实体数。
    pub rendered: usize,
}

/// 单次刷新采集到的调试数据；玩家与世界资源缺失时对应字段为 None。
#[derive(Debug, Clone, PartialEq)]
pub struct DebugOverlayData {
    /// 帧率滑动平均。
    pub fps: f32,
    /// 单帧耗时滑动平均（毫秒）。
    pub frame_ms: f32,
    /// 玩家权威坐标。
    pub position: Option<Vec3>,
    /// 玩家朝向。
    pub facing: Option<FacingInfo>,
    /// 玩家所在区块信息。
    pub chunk: Option<ChunkInfo>,
    /// 世界日历信息。
    pub clock: Option<ClockInfo>,
    /// 世界生成种子。
    pub seed: Option<u32>,
    /// 区块流送计数。
    pub chunk_counts: Option<ChunkCounts>,
    /// 权威模拟刻。
    pub simulation_tick: Option<u64>,
}

/// 把采集数据组装为浮层的多行文本（以换行分隔），文案查本地化表。
pub fn build_lines(data: &DebugOverlayData, localization: &Localization) -> String {
    let mut lines = vec![localization.get("debug.title").to_owned()];
    lines.push(localization.format(
        "debug.fps",
        &[
            ("fps", &format!("{:.0}", data.fps)),
            ("ms", &format!("{:.1}", data.frame_ms)),
        ],
    ));

    match data.position {
        Some(position) => lines.push(localization.format(
            "debug.position",
            &[
                ("x", &format!("{:.1}", position.x)),
                ("y", &format!("{:.1}", position.y)),
                ("z", &format!("{:.1}", position.z)),
            ],
        )),
        None => lines.push(localization.get("debug.position-none").to_owned()),
    }
    match (data.chunk, data.facing) {
        (Some(chunk), facing) => {
            lines.push(localization.format(
                "debug.chunk",
                &[
                    ("x", &chunk.chunk.x.to_string()),
                    ("y", &chunk.chunk.y.to_string()),
                    ("z", &chunk.chunk.z.to_string()),
                    ("lx", &chunk.local.x.to_string()),
                    ("ly", &chunk.local.y.to_string()),
                    ("lz", &chunk.local.z.to_string()),
                ],
            ));
            if let Some(facing) = facing {
                lines.push(localization.format(
                    "debug.facing",
                    &[
                        ("direction", localization.get(facing.direction)),
                        ("yaw", &format!("{:+.1}", facing.yaw_deg)),
                        ("pitch", &format!("{:+.1}", facing.pitch_deg)),
                    ],
                ));
            }
        }
        (None, _) => lines.push(localization.get("debug.chunk-none").to_owned()),
    }
    match data.clock {
        Some(clock) => lines.push(localization.format(
            "debug.time",
            &[
                ("day", &clock.game_day.to_string()),
                ("hour", &format!("{:02}", clock.hour)),
                ("minute", &format!("{:02}", clock.minute)),
                ("season", localization.get(clock.season)),
                ("scale", &format_number(clock.time_scale)),
            ],
        )),
        None => lines.push(localization.get("debug.time-none").to_owned()),
    }
    if let Some(seed) = data.seed {
        lines.push(localization.format("debug.seed", &[("seed", &seed.to_string())]));
    }
    if let Some(counts) = data.chunk_counts {
        lines.push(localization.format(
            "debug.chunk-count",
            &[
                ("loaded", &counts.loaded.to_string()),
                ("expected", &counts.expected.to_string()),
                ("rendered", &counts.rendered.to_string()),
            ],
        ));
    }
    if let Some(tick) = data.simulation_tick {
        lines.push(localization.format("debug.tick", &[("tick", &tick.to_string())]));
    }
    lines.join("\n")
}

/// 时间倍率展示：整数值去掉小数点，避免「2.0 倍」的冗余。
fn format_number(value: f32) -> String {
    if (value - value.round()).abs() < f32::EPSILON {
        format!("{}", value.round() as i64)
    } else {
        format!("{value}")
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/debug_overlay/info.rs"]
mod tests;
