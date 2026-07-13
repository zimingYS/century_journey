use crate::game::world::generation::WorldGenerator;
use crate::game::world::generation::climate::SeasonResource;
use bevy::prelude::*;

/// 统一重导出共享层的世界时间类型。
pub use crate::shared::time::types::{TimeOfDay, TimePhase};

/// 时间流逝系统 — 每帧更新世界时间。
pub fn update_time_system(
    time: Res<Time>,
    mut time_of_day: ResMut<TimeOfDay>,
    season_resource: Res<SeasonResource>,
    mut world_generator: ResMut<WorldGenerator>,
) {
    let game_seconds_per_real_second = time_of_day.speed;
    let delta_hours = time.delta_secs() * game_seconds_per_real_second / 3600.0;

    time_of_day.current_time += delta_hours;
    time_of_day.total_elapsed_hours += delta_hours;
    time_of_day.current_time %= 24.0;

    let season = season_resource.current_season(time_of_day.total_elapsed_hours);
    world_generator.update_season(season);
}
