//! 调试浮层的纯文本组织逻辑：罗盘方位判定、季节标签与行文本组装。
//!
//! 本模块不读取 ECS 资源，数据由调用方采集后传入，便于单元测试。

use crate::game::world::time::Season;
use bevy::math::{IVec3, Vec3};

/// 依据前向向量判定八向罗盘方位。
///
/// 方向约定与主流体素游戏一致：北为 -Z、东为 +X，每 45 度一个扇区。
pub fn compass_direction(forward: Vec3) -> &'static str {
    // 北 = -Z（角度 0），东 = +X（角度 90 度），角度顺时针展开。
    let angle = forward.x.atan2(-forward.z).to_degrees().rem_euclid(360.0) as i32;
    match angle {
        0..=22 | 337.. => "北",
        23..=67 => "东北",
        68..=112 => "东",
        113..=157 => "东南",
        158..=202 => "南",
        203..=247 => "西南",
        248..=292 => "西",
        _ => "西北",
    }
}

/// 返回季节的中文单字标签。
pub fn season_label(season: Season) -> &'static str {
    match season {
        Season::Spring => "春",
        Season::Summer => "夏",
        Season::Autumn => "秋",
        Season::Winter => "冬",
    }
}

/// 玩家朝向信息：罗盘方位与两个欧拉角（角度制）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FacingInfo {
    /// 八向罗盘方位中文名。
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
    /// 季节中文标签。
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

/// 把采集数据组装为浮层的多行文本（以换行分隔）。
pub fn build_lines(data: &DebugOverlayData) -> String {
    let mut lines = vec!["Century Journey 调试（F3 隐藏）".to_owned()];
    lines.push(format!("FPS {:.0}（{:.1} ms/帧）", data.fps, data.frame_ms));

    match data.position {
        Some(position) => lines.push(format!(
            "坐标：({:.1}, {:.1}, {:.1})",
            position.x, position.y, position.z
        )),
        None => lines.push("坐标：暂无玩家实体".to_owned()),
    }
    match (data.chunk, data.facing) {
        (Some(chunk), facing) => {
            lines.push(format!(
                "区块：({}, {}, {}) 局部 ({}, {}, {})",
                chunk.chunk.x,
                chunk.chunk.y,
                chunk.chunk.z,
                chunk.local.x,
                chunk.local.y,
                chunk.local.z
            ));
            if let Some(facing) = facing {
                lines.push(format!(
                    "朝向：{}（偏航 {:+.1}°，俯仰 {:+.1}°）",
                    facing.direction, facing.yaw_deg, facing.pitch_deg
                ));
            }
        }
        (None, _) => lines.push("区块：暂无玩家实体".to_owned()),
    }
    match data.clock {
        Some(clock) => lines.push(format!(
            "时间：第 {} 日 {:02}:{:02}（{}，倍率 {}）",
            clock.game_day, clock.hour, clock.minute, clock.season, clock.time_scale
        )),
        None => lines.push("时间：世界时钟未就绪".to_owned()),
    }
    if let Some(seed) = data.seed {
        lines.push(format!("种子：{seed}"));
    }
    if let Some(counts) = data.chunk_counts {
        lines.push(format!(
            "区块计数：已加载 {}，预期 {}，已渲染 {}",
            counts.loaded, counts.expected, counts.rendered
        ));
    }
    if let Some(tick) = data.simulation_tick {
        lines.push(format!("模拟刻：{tick}"));
    }
    lines.join("\n")
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/ui/debug_overlay/info.rs"]
mod tests;
