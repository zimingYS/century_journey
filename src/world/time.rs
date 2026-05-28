use bevy::prelude::*;

/// 世界时间
#[derive(Resource)]
pub struct TimeOfDay {
    /// 当前世界时间
    pub current_time: f32,
    /// 时间流逝速度
    pub speed: f32,
}

impl Default for TimeOfDay {
    fn default() -> Self {
        Self {
            current_time: 12.0,
            speed: 60.0,
        }
    }
}


pub fn update_time_system(
    time: Res<Time>,
    mut time_of_day: ResMut<TimeOfDay>,
) {
    let game_seconds_per_real_second = time_of_day.speed * 60.0;
    time_of_day.current_time += time.delta_secs() * game_seconds_per_real_second / 3600.0;

    time_of_day.current_time %= 24.0;
}