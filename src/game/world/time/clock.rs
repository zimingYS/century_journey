//! 推进固定步世界时钟，并把累计刻转换为游戏内分钟。

use super::calendar::{
    CalendarSnapshot, DAYS_PER_GAME_YEAR, DAYS_PER_SOLAR_TERM, MINUTES_PER_GAME_DAY,
    MINUTES_PER_GAME_HOUR, SOLAR_TERMS_PER_SEASON, snapshot_at,
};
use super::events::{
    GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed, SeasonChanged,
    SolarTermChanged,
};
use bevy::prelude::{MessageWriter, ResMut, Resource};

/// 世界权威模拟时间。
///
/// 时间只允许在固定步中推进；存档使用三个字段恢复，避免浮点时间累积误差。
#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WorldSimulationClock {
    /// 自会话时钟起点累计的固定步数。
    simulation_tick: u64,
    /// 自日历起点累计的完整游戏分钟。
    game_minute: u64,
    /// 当前游戏分钟内尚未进位的固定步余数。
    subminute_tick: u32,
}

impl Default for WorldSimulationClock {
    fn default() -> Self {
        Self {
            simulation_tick: 0,
            game_minute: NEW_WORLD_START_MINUTE,
            subminute_tick: 0,
        }
    }
}

impl WorldSimulationClock {
    /// 从存档字段恢复时钟，并规范化跨分钟的剩余 Tick。
    pub fn from_persisted(simulation_tick: u64, game_minute: u64, subminute_tick: u32) -> Self {
        let overflow_minutes = subminute_tick as u64 / TICKS_PER_GAME_MINUTE;
        Self {
            simulation_tick,
            game_minute: game_minute.saturating_add(overflow_minutes),
            subminute_tick: (subminute_tick as u64 % TICKS_PER_GAME_MINUTE) as u32,
        }
    }

    /// 从旧版浮点小时字段迁移为固定步时钟。
    pub fn from_legacy_time_of_day(time_of_day: f32) -> Self {
        let normalized = if time_of_day.is_finite() {
            time_of_day.rem_euclid(24.0)
        } else {
            8.0
        };
        Self {
            game_minute: (normalized as f64 * MINUTES_PER_GAME_HOUR as f64).round() as u64
                % MINUTES_PER_GAME_DAY,
            ..Self::default()
        }
    }

    /// 返回自世界创建起单调递增的模拟 Tick。
    pub fn simulation_tick(&self) -> u64 {
        self.simulation_tick
    }

    /// 返回自世界创建起累计的游戏分钟。
    pub fn total_game_minutes(&self) -> u64 {
        self.game_minute
    }

    /// 返回当前游戏分钟内已经过的固定 Tick。
    pub fn subminute_tick(&self) -> u32 {
        self.subminute_tick
    }

    /// 推导当前时钟对应的日历快照。
    pub fn snapshot(&self) -> CalendarSnapshot {
        snapshot_at(self.simulation_tick, self.game_minute)
    }

    /// 计算供渲染帧插值使用的连续小时数。
    pub fn visual_hour(&self, fixed_overstep_fraction: f32) -> f32 {
        let partial_minute = (self.subminute_tick as f32 + fixed_overstep_fraction.clamp(0.0, 1.0))
            / TICKS_PER_GAME_MINUTE as f32;
        ((self.game_minute % MINUTES_PER_GAME_DAY) as f32 + partial_minute)
            / MINUTES_PER_GAME_HOUR as f32
    }

    /// 推进权威时钟并统计跨越的日历边界。
    pub fn advance_ticks(&mut self, ticks: u64) -> ClockAdvance {
        if ticks == 0 {
            return ClockAdvance::default();
        }
        let previous_minute = self.game_minute;
        self.simulation_tick = self.simulation_tick.saturating_add(ticks);
        let accumulated_subminute = self.subminute_tick as u64 + ticks;
        self.game_minute = self
            .game_minute
            .saturating_add(accumulated_subminute / TICKS_PER_GAME_MINUTE);
        self.subminute_tick = (accumulated_subminute % TICKS_PER_GAME_MINUTE) as u32;
        boundary_counts(previous_minute, self.game_minute)
    }
}

/// 权威游戏规则每秒执行的固定步数。
pub const SIMULATION_TICKS_PER_SECOND: u32 = 20;
/// 每经过多少固定步推进一个游戏分钟。
pub const TICKS_PER_GAME_MINUTE: u64 = 20;
/// 一个完整游戏日对应的固定步数。
pub const TICKS_PER_GAME_DAY: u64 = TICKS_PER_GAME_MINUTE * MINUTES_PER_GAME_DAY;
/// 新世界从上午八点开始，避免玩家初次进入时立即面对黑夜。
pub(crate) const NEW_WORLD_START_MINUTE: u64 = 8 * MINUTES_PER_GAME_HOUR;

/// 一次时钟推进跨越的各级日历边界数量。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClockAdvance {
    /// 本次推进跨越的完整游戏分钟数。
    pub game_minutes: u64,
    /// 本次推进跨越的完整游戏小时边界数。
    pub game_hours: u64,
    /// 本次推进跨越的游戏日边界数。
    pub game_days: u64,
    /// 本次推进跨越的节气边界数。
    pub solar_terms: u64,
    /// 本次推进跨越的季节边界数。
    pub seasons: u64,
    /// 本次推进跨越的游戏年边界数。
    pub years: u64,
}

impl ClockAdvance {
    /// 合并另一段连续推进产生的边界计数。
    pub fn accumulate(&mut self, other: Self) {
        self.game_minutes += other.game_minutes;
        self.game_hours += other.game_hours;
        self.game_days += other.game_days;
        self.solar_terms += other.solar_terms;
        self.seasons += other.seasons;
        self.years += other.years;
    }
}

fn boundary_counts(previous_minute: u64, current_minute: u64) -> ClockAdvance {
    let boundary_delta = |period: u64| current_minute / period - previous_minute / period;
    let minutes_per_term = MINUTES_PER_GAME_DAY * DAYS_PER_SOLAR_TERM;
    let minutes_per_season = minutes_per_term * SOLAR_TERMS_PER_SEASON;
    let minutes_per_year = MINUTES_PER_GAME_DAY * DAYS_PER_GAME_YEAR;
    ClockAdvance {
        game_minutes: current_minute - previous_minute,
        game_hours: boundary_delta(MINUTES_PER_GAME_HOUR),
        game_days: boundary_delta(MINUTES_PER_GAME_DAY),
        solar_terms: boundary_delta(minutes_per_term),
        seasons: boundary_delta(minutes_per_season),
        years: boundary_delta(minutes_per_year),
    }
}

/// 在固定步推进时钟，并向其他玩法模块发送已跨越的日历边界消息。
pub fn advance_world_simulation_clock(
    mut clock: ResMut<WorldSimulationClock>,
    mut minute_events: MessageWriter<GameMinuteElapsed>,
    mut hour_events: MessageWriter<GameHourElapsed>,
    mut day_events: MessageWriter<GameDayElapsed>,
    mut solar_term_events: MessageWriter<SolarTermChanged>,
    mut season_events: MessageWriter<SeasonChanged>,
    mut year_events: MessageWriter<GameYearElapsed>,
) {
    let crossed = clock.advance_ticks(1);
    if crossed.game_minutes == 0 {
        return;
    }
    let snapshot = clock.snapshot();
    minute_events.write(GameMinuteElapsed(snapshot));
    if crossed.game_hours > 0 {
        hour_events.write(GameHourElapsed(snapshot));
    }
    if crossed.game_days > 0 {
        day_events.write(GameDayElapsed(snapshot));
    }
    if crossed.solar_terms > 0 {
        solar_term_events.write(SolarTermChanged(snapshot));
    }
    if crossed.seasons > 0 {
        season_events.write(SeasonChanged(snapshot));
    }
    if crossed.years > 0 {
        year_events.write(GameYearElapsed(snapshot));
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/time/clock.rs"]
mod tests;
