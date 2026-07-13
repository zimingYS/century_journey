use bevy::prelude::*;

use super::{SUNRISE_END, SUNRISE_START, SUNSET_END, SUNSET_START};

/// 世界时间 — Client（天空渲染）和 Server（世界模拟）共享。
#[derive(Resource)]
pub struct TimeOfDay {
    /// 当前世界时间（0.0 ~ 24.0 小时）
    pub current_time: f32,
    /// 累计经过的游戏小时数，用于季节等长期计时
    pub total_elapsed_hours: f32,
    /// 时间流逝速度（游戏秒/现实秒）
    pub speed: f32,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            current_time: 8.0,
            total_elapsed_hours: 0.0,
            speed: 60.0,
        }
    }
}

impl TimeOfDay {
    /// 获取当前时间阶段
    pub fn phase(&self) -> TimePhase {
        let t = self.current_time;

        if (SUNRISE_START..SUNRISE_END).contains(&t) {
            TimePhase::Sunrise
        } else if (SUNRISE_END..SUNSET_START).contains(&t) {
            TimePhase::Day
        } else if (SUNSET_START..SUNSET_END).contains(&t) {
            TimePhase::Sunset
        } else {
            TimePhase::Night
        }
    }

    /// 获取日出/日落过渡因子 (0.0=夜晚端, 1.0=白天端)
    pub fn twilight_factor(&self) -> f32 {
        let t = self.current_time;

        if (SUNRISE_START..SUNRISE_END).contains(&t) {
            (t - SUNRISE_START) / (SUNRISE_END - SUNRISE_START)
        } else if (SUNRISE_END..SUNSET_START).contains(&t) {
            1.0
        } else if (SUNSET_START..SUNSET_END).contains(&t) {
            1.0 - (t - SUNSET_START) / (SUNSET_END - SUNSET_START)
        } else {
            0.0
        }
    }

    /// 获取夜晚因子 (0.0=白天, 1.0=深夜)
    pub fn night_factor(&self) -> f32 {
        1.0 - self.twilight_factor()
    }
}

/// 时间阶段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TimePhase {
    Night,
    Sunrise,
    Day,
    Sunset,
}
