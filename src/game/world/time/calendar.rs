//! 定义季节、节气及由游戏分钟推导日历快照的规则。

use serde::{Deserialize, Serialize};

/// 每个游戏小时包含的游戏分钟数。
pub const MINUTES_PER_GAME_HOUR: u64 = 60;
/// 每个游戏日包含的游戏小时数。
pub const HOURS_PER_GAME_DAY: u64 = 24;
/// 每个节气持续的游戏日数。
pub const DAYS_PER_SOLAR_TERM: u64 = 2;
/// 每个季节包含的节气数。
pub const SOLAR_TERMS_PER_SEASON: u64 = 6;
/// 每个游戏年包含的季节数。
pub const SEASONS_PER_YEAR: u64 = 4;
/// 每个游戏年包含的节气总数。
pub const SOLAR_TERMS_PER_YEAR: u64 = SOLAR_TERMS_PER_SEASON * SEASONS_PER_YEAR;
/// 每个游戏年包含的游戏日总数。
pub const DAYS_PER_GAME_YEAR: u64 = DAYS_PER_SOLAR_TERM * SOLAR_TERMS_PER_YEAR;
/// 每个游戏日包含的游戏分钟总数。
pub const MINUTES_PER_GAME_DAY: u64 = MINUTES_PER_GAME_HOUR * HOURS_PER_GAME_DAY;

/// 世界日历使用的四季。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

/// 世界日历使用的二十四节气。
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
    /// 按一个游戏年中的出现顺序排列全部节气。
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

    /// 返回当前节气所属的季节。
    pub fn season(self) -> Season {
        match Self::ALL.iter().position(|term| *term == self).unwrap_or(0) / 6 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}

/// 某个权威模拟时刻对应的完整日历读模型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarSnapshot {
    /// 生成该快照时的权威固定步序号。
    pub simulation_tick: u64,
    /// 当前小时内的分钟，范围为 0 到 59。
    pub minute: u8,
    /// 当前游戏日内的小时，范围为 0 到 23。
    pub hour: u8,
    /// 自世界创建起从一开始计数的累计游戏日。
    pub game_day: u64,
    /// 当前游戏年内从一开始计数的日期。
    pub day_of_year: u16,
    /// 当前日期所属的节气。
    pub solar_term: SolarTerm,
    /// 当前节气对应的季节。
    pub season: Season,
    /// 自世界创建起从一开始计数的游戏年。
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
