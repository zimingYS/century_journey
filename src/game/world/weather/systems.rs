//! 固定步天气系统：重算派生温湿度，并在游戏小时边界演化马尔可夫状态。

use crate::game::player::identity::Player;
use crate::game::simulation::SimulationRng;
use crate::game::world::generation::terrain::climate::ClimateSampler;
use crate::game::world::time::{GameHourElapsed, WorldSimulationClock};
use crate::game::world::weather::cell::WeatherCell;
use crate::game::world::weather::evolution::{
    compute_humidity, compute_temperature_c, step_weather,
};
use bevy::prelude::*;

/// 天气随机流使用的领域标识（任意稳定常量，避免与其他领域冲突）。
const WEATHER_RNG_DOMAIN: u64 = 0x0057_4541_5448_4552;

/// 固定步天气系统：每 tick 重算派生温度与湿度；每个游戏小时边界演化云量。
pub fn weather_evolve_system(
    clock: Res<WorldSimulationClock>,
    climate: Res<ClimateSampler>,
    rng: Res<SimulationRng>,
    mut hour_events: MessageReader<GameHourElapsed>,
    mut weather: ResMut<WeatherCell>,
    player: Query<&Transform, With<Player>>,
) {
    let snapshot = clock.snapshot();
    let season = snapshot.season;
    let hour = snapshot.hour;

    // 全局单 cell 跟随玩家位置采样气候带；无玩家时回退到原点。
    let (world_x, world_z) = player
        .single()
        .map(|transform| {
            (
                transform.translation.x as i32,
                transform.translation.z as i32,
            )
        })
        .unwrap_or((0, 0));

    let base_temperature = climate.sample_temperature_with_season(world_x, world_z, season);
    let base_humidity = climate.sample_humidity_with_season(world_x, world_z, season);

    // 循环三件套：温度（用当前云量/降水）→ 湿度（用刚算温度）→ 云量演化（用湿度）。
    weather.temperature_c = compute_temperature_c(
        base_temperature,
        weather.cloud_water,
        weather.precipitation,
        hour,
    );
    weather.humidity =
        compute_humidity(base_humidity, weather.temperature_c, weather.precipitation);

    // 天气维度只在跨小时边界时转移一次，避免同一小时内反复掷骰。
    if hour_events.read().count() > 0 {
        let game_hour =
            clock.total_game_minutes() / crate::game::world::time::MINUTES_PER_GAME_HOUR;
        let mut hour_rng = rng.for_event(WEATHER_RNG_DOMAIN, game_hour, 0);
        step_weather(&mut weather, &mut hour_rng, season);
    }
}
