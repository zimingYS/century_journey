//! 把 Game 层权威天气（`WeatherCell`）映射到云场表现状态。
//!
//! 纯表现层桥接：在渲染帧读取权威天气，把云量与雾霾折算成云场的不透明度与
//! 能见度参数。不进入 FixedUpdate，不参与权威模拟。

use crate::client::sky::cloud::components::CloudWeatherState;
use crate::game::world::weather::WeatherCell;
use bevy::prelude::*;

/// 在渲染帧把权威天气映射为云场表现参数。
pub fn sync_weather_to_cloud_system(
    weather: Option<Res<WeatherCell>>,
    mut cloud_state: ResMut<CloudWeatherState>,
) {
    let Some(weather) = weather else {
        return;
    };
    cloud_state.coverage = weather.cloud_water;
    // 雾霾降低远景能见度：满雾（1.0）降至 0.4 能见度，无雾保持 1.0。
    cloud_state.visibility = (1.0 - weather.fog_density * 0.6).clamp(0.0, 1.0);
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/sky/cloud/weather_adapter.rs"]
mod tests;
