//! 将权威世界时钟转换成天空渲染所需的连续时刻。

use bevy::prelude::{Fixed, Res, ResMut, Resource, Time};

use crate::game::world::time::WorldSimulationClock;

/// 客户端尚未收到首个模拟时钟结果时使用的初始小时。
const INITIAL_VISUAL_HOUR: f32 = 8.0;
/// 日出开始时间，单位为游戏小时。
const SUNRISE_START: f32 = 5.0;
/// 日出结束时间，单位为游戏小时。
const SUNRISE_END: f32 = 7.0;
/// 日落开始时间，单位为游戏小时。
const SUNSET_START: f32 = 17.0;
/// 日落结束时间，单位为游戏小时。
const SUNSET_END: f32 = 19.0;

/// 当前渲染帧使用的连续世界时刻，范围为 0 到 24 小时。
///
/// 该资源是客户端表现快照，不能反向参与权威模拟或存档。
#[derive(Resource, Debug)]
pub struct TimeOfDay {
    /// 当前连续游戏小时。
    pub current_time: f32,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            current_time: INITIAL_VISUAL_HOUR,
        }
    }
}

impl TimeOfDay {
    /// 返回当前昼夜阶段。
    pub fn phase(&self) -> TimePhase {
        let hour = self.current_time;
        if (SUNRISE_START..SUNRISE_END).contains(&hour) {
            TimePhase::Sunrise
        } else if (SUNRISE_END..SUNSET_START).contains(&hour) {
            TimePhase::Day
        } else if (SUNSET_START..SUNSET_END).contains(&hour) {
            TimePhase::Sunset
        } else {
            TimePhase::Night
        }
    }

    /// 返回日出或日落过渡因子，其中 0 表示夜晚端，1 表示白天端。
    pub fn twilight_factor(&self) -> f32 {
        let hour = self.current_time;
        if (SUNRISE_START..SUNRISE_END).contains(&hour) {
            (hour - SUNRISE_START) / (SUNRISE_END - SUNRISE_START)
        } else if (SUNRISE_END..SUNSET_START).contains(&hour) {
            1.0
        } else if (SUNSET_START..SUNSET_END).contains(&hour) {
            1.0 - (hour - SUNSET_START) / (SUNSET_END - SUNSET_START)
        } else {
            0.0
        }
    }

    /// 返回夜晚权重，其中 0 表示白天，1 表示深夜。
    pub fn night_factor(&self) -> f32 {
        1.0 - self.twilight_factor()
    }
}

/// 天空渲染使用的粗粒度昼夜阶段。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimePhase {
    Night,
    Sunrise,
    Day,
    Sunset,
}

/// 使用固定步余量插值当前渲染时刻。
///
/// 本系统只读取权威时钟并写入客户端表现资源，调度在 PreUpdate。
pub(super) fn update_visual_time(
    clock: Res<WorldSimulationClock>,
    fixed_time: Res<Time<Fixed>>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    time_of_day.current_time = clock.visual_hour(fixed_time.overstep_fraction());
}

#[cfg(test)]
#[path = "../../../tests/unit/client/presentation/time.rs"]
mod tests;
