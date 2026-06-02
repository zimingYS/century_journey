use bevy::prelude::*;

/// 世界时间
#[derive(Resource)]
pub struct TimeOfDay {
    /// 当前世界时间
    pub current_time: f32,
    /// 时间流逝速度
    pub speed: f32,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            current_time: 24.0,
            speed: 60.0,
        }
    }
}

impl TimeOfDay {
    /// 获取当前时间阶段
    pub fn phase(&self) -> TimePhase {
        use crate::core::constant::sky::*;
        let t = self.current_time;

        if t >= SUNRISE_START && t < SUNRISE_END {
            TimePhase::Sunrise
        } else if t >= SUNRISE_END && t < SUNSET_START {
            TimePhase::Day
        } else if t >= SUNSET_START && t < SUNSET_END {
            TimePhase::Sunset
        } else {
            TimePhase::Night
        }
    }

    /// 获取日出/日落过渡因子 (0.0=夜晚端, 1.0=白天端)
    /// 在非过渡时段返回稳定的 0.0 或 1.0
    pub fn twilight_factor(&self) -> f32 {
        use crate::core::constant::sky::*;
        let t = self.current_time;

        if t >= SUNRISE_START && t < SUNRISE_END {
            // 日出：从 0.0 渐变到 1.0
            (t - SUNRISE_START) / (SUNRISE_END - SUNRISE_START)
        } else if t >= SUNRISE_END && t < SUNSET_START {
            1.0 // 白天
        } else if t >= SUNSET_START && t < SUNSET_END {
            // 日落：从 1.0 渐变到 0.0
            1.0 - (t - SUNSET_START) / (SUNSET_END - SUNSET_START)
        } else {
            0.0 // 夜晚
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
    /// 夜晚（日落后 ~ 日出前）
    Night,
    /// 日出（黎明过渡）
    Sunrise,
    /// 白天
    Day,
    /// 日落（黄昏过渡）
    Sunset,
}


pub fn update_time_system(
    time: Res<Time>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    let game_seconds_per_real_second = time_of_day.speed;
    time_of_day.current_time += time.delta_secs() * game_seconds_per_real_second / 3600.0;

    time_of_day.current_time %= 24.0;
}