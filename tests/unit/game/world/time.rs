use super::*;

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
