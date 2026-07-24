use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub const SIMULATION_TICKS_PER_SECOND: u32 = 20;
pub const TICKS_PER_GAME_MINUTE: u64 = 20;
pub const MINUTES_PER_GAME_HOUR: u64 = 60;
pub const HOURS_PER_GAME_DAY: u64 = 24;
pub const DAYS_PER_SOLAR_TERM: u64 = 2;
pub const SOLAR_TERMS_PER_SEASON: u64 = 6;
pub const SEASONS_PER_YEAR: u64 = 4;
pub const SOLAR_TERMS_PER_YEAR: u64 = SOLAR_TERMS_PER_SEASON * SEASONS_PER_YEAR;
pub const DAYS_PER_GAME_YEAR: u64 = DAYS_PER_SOLAR_TERM * SOLAR_TERMS_PER_YEAR;
pub const MINUTES_PER_GAME_DAY: u64 = MINUTES_PER_GAME_HOUR * HOURS_PER_GAME_DAY;
pub const TICKS_PER_GAME_DAY: u64 = TICKS_PER_GAME_MINUTE * MINUTES_PER_GAME_DAY;

const NEW_WORLD_START_MINUTE: u64 = 8 * MINUTES_PER_GAME_HOUR;

/// 统一重导出共享层的世界时间类型。
pub use crate::shared::time::types::{TimeOfDay, TimePhase};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SolarTerm {
    BeginningOfSpring,
    RainWater,
    AwakeningOfInsects,
    SpringEquinox,
    ClearAndBright,
    GrainRain,
    BeginningOfSummer,
    GrainBuds,
    GrainInEar,
    SummerSolstice,
    MinorHeat,
    MajorHeat,
    BeginningOfAutumn,
    EndOfHeat,
    WhiteDew,
    AutumnEquinox,
    ColdDew,
    FrostDescent,
    BeginningOfWinter,
    MinorSnow,
    MajorSnow,
    WinterSolstice,
    MinorCold,
    MajorCold,
}

impl SolarTerm {
    pub const ALL: [Self; 24] = [
        Self::BeginningOfSpring,
        Self::RainWater,
        Self::AwakeningOfInsects,
        Self::SpringEquinox,
        Self::ClearAndBright,
        Self::GrainRain,
        Self::BeginningOfSummer,
        Self::GrainBuds,
        Self::GrainInEar,
        Self::SummerSolstice,
        Self::MinorHeat,
        Self::MajorHeat,
        Self::BeginningOfAutumn,
        Self::EndOfHeat,
        Self::WhiteDew,
        Self::AutumnEquinox,
        Self::ColdDew,
        Self::FrostDescent,
        Self::BeginningOfWinter,
        Self::MinorSnow,
        Self::MajorSnow,
        Self::WinterSolstice,
        Self::MinorCold,
        Self::MajorCold,
    ];

    pub fn season(self) -> Season {
        match Self::ALL.iter().position(|term| *term == self).unwrap_or(0) / 6 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarSnapshot {
    pub simulation_tick: u64,
    pub minute: u8,
    pub hour: u8,
    pub game_day: u64,
    pub day_of_year: u16,
    pub solar_term: SolarTerm,
    pub season: Season,
    pub year: u64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClockAdvance {
    pub game_minutes: u64,
    pub game_hours: u64,
    pub game_days: u64,
    pub solar_terms: u64,
    pub seasons: u64,
    pub years: u64,
}

impl ClockAdvance {
    pub fn accumulate(&mut self, other: Self) {
        self.game_minutes += other.game_minutes;
        self.game_hours += other.game_hours;
        self.game_days += other.game_days;
        self.solar_terms += other.solar_terms;
        self.seasons += other.seasons;
        self.years += other.years;
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WorldSimulationClock {
    simulation_tick: u64,
    game_minute: u64,
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
    pub fn from_persisted(simulation_tick: u64, game_minute: u64, subminute_tick: u32) -> Self {
        let overflow_minutes = subminute_tick as u64 / TICKS_PER_GAME_MINUTE;
        Self {
            simulation_tick,
            game_minute: game_minute.saturating_add(overflow_minutes),
            subminute_tick: (subminute_tick as u64 % TICKS_PER_GAME_MINUTE) as u32,
        }
    }

    pub fn from_legacy_time_of_day(time_of_day: f32) -> Self {
        let normalized = if time_of_day.is_finite() {
            time_of_day.rem_euclid(24.0)
        } else {
            8.0
        };
        Self {
            game_minute: (normalized as f64 * MINUTES_PER_GAME_HOUR as f64).round() as u64
                % MINUTES_PER_GAME_DAY,
            ..default()
        }
    }

    pub fn simulation_tick(&self) -> u64 {
        self.simulation_tick
    }

    pub fn total_game_minutes(&self) -> u64 {
        self.game_minute
    }

    pub fn subminute_tick(&self) -> u32 {
        self.subminute_tick
    }

    pub fn snapshot(&self) -> CalendarSnapshot {
        snapshot_at(self.simulation_tick, self.game_minute)
    }

    pub fn visual_hour(&self, fixed_overstep_fraction: f32) -> f32 {
        let partial_minute = (self.subminute_tick as f32 + fixed_overstep_fraction.clamp(0.0, 1.0))
            / TICKS_PER_GAME_MINUTE as f32;
        ((self.game_minute % MINUTES_PER_GAME_DAY) as f32 + partial_minute)
            / MINUTES_PER_GAME_HOUR as f32
    }

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

fn snapshot_at(simulation_tick: u64, game_minute: u64) -> CalendarSnapshot {
    let minute_of_day = game_minute % MINUTES_PER_GAME_DAY;
    let absolute_day = game_minute / MINUTES_PER_GAME_DAY;
    let day_of_year_zero = absolute_day % DAYS_PER_GAME_YEAR;
    let solar_term = SolarTerm::ALL[(day_of_year_zero / DAYS_PER_SOLAR_TERM) as usize];
    CalendarSnapshot {
        simulation_tick,
        minute: (minute_of_day % MINUTES_PER_GAME_HOUR) as u8,
        hour: (minute_of_day / MINUTES_PER_GAME_HOUR) as u8,
        game_day: absolute_day + 1,
        day_of_year: day_of_year_zero as u16 + 1,
        solar_term,
        season: solar_term.season(),
        year: absolute_day / DAYS_PER_GAME_YEAR + 1,
    }
}

macro_rules! clock_message {
    ($name:ident) => {
        #[derive(Message, Debug, Clone, Copy)]
        pub struct $name(pub CalendarSnapshot);
    };
}

clock_message!(GameMinuteElapsed);
clock_message!(GameHourElapsed);
clock_message!(GameDayElapsed);
clock_message!(SolarTermChanged);
clock_message!(SeasonChanged);
clock_message!(GameYearElapsed);

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

pub fn update_visual_time(
    clock: Res<WorldSimulationClock>,
    fixed_time: Res<Time<Fixed>>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    time_of_day.current_time = clock.visual_hour(fixed_time.overstep_fraction());
}

#[cfg(test)]
#[path = "../../../tests/unit/game/world/time.rs"]
mod tests;
