use serde::{Deserialize, Serialize};

pub const MINUTES_PER_GAME_HOUR: u64 = 60;
pub const HOURS_PER_GAME_DAY: u64 = 24;
pub const DAYS_PER_SOLAR_TERM: u64 = 2;
pub const SOLAR_TERMS_PER_SEASON: u64 = 6;
pub const SEASONS_PER_YEAR: u64 = 4;
pub const SOLAR_TERMS_PER_YEAR: u64 = SOLAR_TERMS_PER_SEASON * SEASONS_PER_YEAR;
pub const DAYS_PER_GAME_YEAR: u64 = DAYS_PER_SOLAR_TERM * SOLAR_TERMS_PER_YEAR;
pub const MINUTES_PER_GAME_DAY: u64 = MINUTES_PER_GAME_HOUR * HOURS_PER_GAME_DAY;

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

/// 根据权威模拟时间推导日历快照。
///
/// 该函数不读取 ECS 资源，保证存档恢复和固定步模拟使用相同的日历规则。
pub(super) fn snapshot_at(simulation_tick: u64, game_minute: u64) -> CalendarSnapshot {
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
