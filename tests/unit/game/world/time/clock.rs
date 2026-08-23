use super::*;
use crate::game::gameplay::rules::GameRules;
use crate::game::world::time::{
    GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed, MINUTES_PER_GAME_DAY,
    SEASONS_PER_YEAR, SOLAR_TERMS_PER_YEAR, Season, SeasonChanged, SolarTerm, SolarTermChanged,
};
use bevy::prelude::{App, FixedUpdate};

#[test]
fn calendar_boundaries_follow_the_24_solar_terms() {
    let mut clock = WorldSimulationClock::default();
    let crossed = clock.advance_ticks(TICKS_PER_GAME_DAY * DAYS_PER_GAME_YEAR);
    let snapshot = clock.snapshot();

    assert_eq!(snapshot.year, 2);
    assert_eq!(snapshot.day_of_year, 1);
    assert_eq!(snapshot.solar_term, SolarTerm::BeginningOfSpring);
    assert_eq!(snapshot.season, Season::Spring);
    assert_eq!(crossed.game_days, DAYS_PER_GAME_YEAR);
    assert_eq!(crossed.solar_terms, SOLAR_TERMS_PER_YEAR);
    assert_eq!(crossed.seasons, SEASONS_PER_YEAR);
    assert_eq!(crossed.years, 1);
}

#[test]
fn different_render_rates_simulate_one_hundred_days_identically() {
    fn simulate(fps: u64) -> (WorldSimulationClock, ClockAdvance) {
        let target_ticks = TICKS_PER_GAME_DAY * 100;
        let mut clock = WorldSimulationClock::default();
        let mut frame_remainder = 0u64;
        let mut crossed = ClockAdvance::default();
        while clock.simulation_tick() < target_ticks {
            // Each harness second represents one game day; only frame grouping varies.
            frame_remainder += TICKS_PER_GAME_DAY;
            let ticks = (frame_remainder / fps).min(target_ticks - clock.simulation_tick());
            frame_remainder %= fps;
            crossed.accumulate(clock.advance_ticks(ticks));
        }
        (clock, crossed)
    }

    let (at_30, events_30) = simulate(30);
    let (at_60, events_60) = simulate(60);
    let (at_144, events_144) = simulate(144);

    assert_eq!(at_30, at_60);
    assert_eq!(at_60, at_144);
    assert_eq!(events_30, events_60);
    assert_eq!(events_60, events_144);
    assert_eq!(at_30.snapshot().game_day, 101);
}

#[test]
fn persisted_subminute_ticks_are_normalized() {
    let clock = WorldSimulationClock::from_persisted(10, 100, 45);
    assert_eq!(clock.total_game_minutes(), 102);
    assert_eq!(clock.subminute_tick(), 5);
}

#[test]
fn set_time_of_day_keeps_the_current_game_day() {
    let mut clock = WorldSimulationClock::default();
    clock.advance_ticks(TICKS_PER_GAME_DAY * 3 + TICKS_PER_GAME_MINUTE * 10);
    clock.set_time_of_day(0);
    assert_eq!(clock.total_game_minutes(), 3 * MINUTES_PER_GAME_DAY);
    let snapshot = clock.snapshot();
    assert_eq!(snapshot.game_day, 4);
    assert_eq!(snapshot.hour, 0);
    assert_eq!(snapshot.minute, 0);
}

#[test]
fn set_time_of_day_resets_subminute_remainder_and_keeps_advancing() {
    let mut clock = WorldSimulationClock::default();
    clock.advance_ticks(TICKS_PER_GAME_MINUTE + 7);
    clock.set_time_of_day(600);
    assert_eq!(clock.subminute_tick(), 0);
    assert_eq!(clock.total_game_minutes(), 600);
    let crossed = clock.advance_ticks(TICKS_PER_GAME_MINUTE);
    assert_eq!(crossed.game_minutes, 1);
    let snapshot = clock.snapshot();
    assert_eq!(snapshot.hour, 10);
    assert_eq!(snapshot.minute, 1);
}

#[test]
fn set_time_of_day_wraps_out_of_range_minute_into_the_day() {
    let mut clock = WorldSimulationClock::default();
    clock.set_time_of_day(MINUTES_PER_GAME_DAY + 30);
    assert_eq!(clock.total_game_minutes(), 30);
    assert_eq!(clock.snapshot().hour, 0);
    assert_eq!(clock.snapshot().minute, 30);
}

/// 构造挂载了时钟推进系统的最小应用，用于按固定步驱动真实系统。
fn clock_app_with_scale(time_scale: f32) -> App {
    let mut app = App::new();
    app.insert_resource(GameRules { time_scale })
        .init_resource::<WorldSimulationClock>()
        .add_message::<GameMinuteElapsed>()
        .add_message::<GameHourElapsed>()
        .add_message::<GameDayElapsed>()
        .add_message::<SolarTermChanged>()
        .add_message::<SeasonChanged>()
        .add_message::<GameYearElapsed>()
        .add_systems(FixedUpdate, advance_world_simulation_clock);
    app
}

#[test]
fn advance_fixed_step_advances_one_simulation_tick_per_step() {
    let mut clock = WorldSimulationClock::default();
    let crossed = clock.advance_fixed_step(TICKS_PER_GAME_MINUTE * 2);
    assert_eq!(clock.simulation_tick(), 1);
    assert_eq!(crossed.game_minutes, 2);
    assert_eq!(clock.total_game_minutes(), NEW_WORLD_START_MINUTE + 2);
}

#[test]
fn time_scale_accelerates_calendar_without_distorting_simulation_ticks() {
    let mut app = clock_app_with_scale(10.0);
    for _ in 0..2 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let clock = app.world().resource::<WorldSimulationClock>();
    // 模拟刻按真实固定步推进，玩家命令调度不受倍率影响。
    assert_eq!(clock.simulation_tick(), 2);
    // 日历按倍率加速：2 步 × 10 刻 = 1 个游戏分钟。
    assert_eq!(clock.total_game_minutes(), NEW_WORLD_START_MINUTE + 1);
}

#[test]
fn paused_time_still_advances_simulation_ticks() {
    let mut app = clock_app_with_scale(0.0);
    for _ in 0..3 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let clock = app.world().resource::<WorldSimulationClock>();
    // 倍率为 0 只暂停日历；模拟节拍继续，输入命令不中断。
    assert_eq!(clock.simulation_tick(), 3);
    assert_eq!(clock.total_game_minutes(), NEW_WORLD_START_MINUTE);
}

#[test]
fn fractional_time_scale_accumulates_across_fixed_steps() {
    let mut app = clock_app_with_scale(0.5);
    for _ in 0..2 {
        app.world_mut().run_schedule(FixedUpdate);
    }
    let clock = app.world().resource::<WorldSimulationClock>();
    assert_eq!(clock.simulation_tick(), 2);
    // 两步各积累 0.5 刻，合计推进 1 个日历刻，尚未进位为分钟。
    assert_eq!(clock.subminute_tick(), 1);
    assert_eq!(clock.total_game_minutes(), NEW_WORLD_START_MINUTE);
}
