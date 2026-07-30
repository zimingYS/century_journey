//! 组装权威世界时钟与日历事件。

use bevy::prelude::*;

use super::{
    GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed,
    SIMULATION_TICKS_PER_SECOND, SeasonChanged, SolarTermChanged, WorldSimulationClock,
    advance_world_simulation_clock,
};
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;

/// 注册固定步世界时钟及其派生日历事件。
pub(in crate::game::world) struct WorldTimePlugin;

impl Plugin for WorldTimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldSimulationClock::default())
            .insert_resource(Time::<Fixed>::from_hz(SIMULATION_TICKS_PER_SECOND as f64))
            .add_message::<GameMinuteElapsed>()
            .add_message::<GameHourElapsed>()
            .add_message::<GameDayElapsed>()
            .add_message::<SolarTermChanged>()
            .add_message::<SeasonChanged>()
            .add_message::<GameYearElapsed>()
            .add_systems(
                FixedUpdate,
                advance_world_simulation_clock
                    .in_set(SimulationSet::Clock)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}
