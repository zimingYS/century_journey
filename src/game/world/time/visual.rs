use super::clock::WorldSimulationClock;
pub use crate::shared::time::types::{TimeOfDay, TimePhase};
use bevy::prelude::{Fixed, Res, ResMut, Time};

/// 将固定步权威时钟转换为客户端渲染使用的连续时刻。
///
/// 本系统只写入表现层时间，不能反向修改世界模拟时钟。
pub fn update_visual_time(
    clock: Res<WorldSimulationClock>,
    fixed_time: Res<Time<Fixed>>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    time_of_day.current_time = clock.visual_hour(fixed_time.overstep_fraction());
}
