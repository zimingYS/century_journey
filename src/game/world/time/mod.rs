mod calendar;
mod clock;
mod events;
mod plugin;
mod visual;

pub use calendar::{
    CalendarSnapshot, DAYS_PER_GAME_YEAR, DAYS_PER_SOLAR_TERM, HOURS_PER_GAME_DAY,
    MINUTES_PER_GAME_DAY, MINUTES_PER_GAME_HOUR, SEASONS_PER_YEAR, SOLAR_TERMS_PER_SEASON,
    SOLAR_TERMS_PER_YEAR, Season, SolarTerm,
};
pub use clock::{
    ClockAdvance, SIMULATION_TICKS_PER_SECOND, TICKS_PER_GAME_DAY, TICKS_PER_GAME_MINUTE,
    WorldSimulationClock, advance_world_simulation_clock,
};
pub use events::{
    GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed, SeasonChanged,
    SolarTermChanged,
};
pub use visual::{TimeOfDay, TimePhase, update_visual_time};

pub(in crate::game::world) use plugin::WorldTimePlugin;
