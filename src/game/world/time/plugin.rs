use super::{
    GameDayElapsed, GameHourElapsed, GameMinuteElapsed, GameYearElapsed,
    SIMULATION_TICKS_PER_SECOND, SeasonChanged, SolarTermChanged, TimeOfDay, WorldSimulationClock,
    advance_world_simulation_clock, update_visual_time,
};
use crate::game::simulation::SimulationSet;
use crate::shared::states::AppState;
use bevy::prelude::*;

/// 组装世界时间、日历事件和客户端视觉时间同步。
///
/// 权威时钟在固定步推进；视觉时间仅在渲染帧读取时钟结果。
pub(in crate::game::world) struct WorldTimePlugin;

impl Plugin for WorldTimePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TimeOfDay::default())
            .insert_resource(WorldSimulationClock::default())
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
            )
            .add_systems(
                PreUpdate,
                update_visual_time.run_if(in_state(AppState::InGame)),
            );
    }
}
