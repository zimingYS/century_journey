//! 世界天气状态（Game 层权威，全局单 cell）。
//!
//! 天气回答"现在天气如何"，与静态气候带（`ClimateSampler`）分层：
//! 气候是种子决定的稳定温湿度带，天气是随游戏时间、按季节与环境概率演进的
//! 马尔可夫状态。温度（°C）与湿度是气候、昼夜、天气三者的派生值。
//!
//! 演进由「季节 + 气候带湿度」做主因，通过确定性随机概率触发状态转移：
//! 夏季多雨、冬季干旱少雨；雨天以固定概率转晴；雾独立于降水。

use crate::game::player::identity::Player;
use crate::game::simulation::SimulationRng;
use crate::game::world::generation::terrain::climate::ClimateSampler;
use crate::game::world::time::{GameHourElapsed, Season, WorldSimulationClock};
use crate::shared::random::RandomSource;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 天气随机流使用的领域标识（任意稳定常量，避免与其他领域冲突）。
const WEATHER_RNG_DOMAIN: u64 = 0x0057_4541_5448_4552;

/// 全局单 cell 天气状态（v1；v2 扩展为 256² 网格）。
///
/// 云量、降水、雾霾是马尔可夫状态（存档）；温度是派生值（运行时计算、不存档）。
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherCell {
    /// 云含水量 0~1（云系统消费）。
    pub cloud_water: f32,
    /// 降水强度 0~1（雨/雪/冰雹表现）。
    pub precipitation: f32,
    /// 雾霾密度 0~1。
    pub fog_density: f32,
    /// 当前温度（°C），由气候、昼夜、天气派生；运行时计算、不存档。
    #[serde(skip)]
    pub temperature_c: f32,
}

impl Default for WeatherCell {
    fn default() -> Self {
        Self {
            cloud_water: 0.3,
            precipitation: 0.0,
            fog_density: 0.0,
            temperature_c: 20.0,
        }
    }
}

/// 昼夜温度修正因子，范围约 -0.87~0.87：14 点最热、2 点最冷。
pub fn diurnal_temperature_factor(hour: u8) -> f32 {
    let t = (hour as f32 - 6.0) / 24.0 * std::f32::consts::TAU;
    t.sin()
}

/// 由气候基础温度（0~1）、当前降水与小时派生温度（°C）。
///
/// 气候 0~1 映射到 -10°C~50°C；昼夜修正 ±6°C；强降水约降温 5°C。
pub fn compute_temperature_c(base_temperature: f64, precipitation: f32, hour: u8) -> f32 {
    let base = base_temperature as f32;
    let diurnal = diurnal_temperature_factor(hour);
    let rain_cooling = -precipitation * 5.0;
    base * 60.0 - 10.0 + diurnal * 6.0 + rain_cooling
}

/// 晴天下雨的概率，由季节与气候带湿度做主因。
///
/// 夏季多雨、冬季干旱少雨；湿度越高的气候带越容易下雨。
pub fn rain_probability(season: Season, humidity: f64) -> f32 {
    let seasonal = match season {
        Season::Spring => 0.30,
        Season::Summer => 0.45,
        Season::Autumn => 0.20,
        Season::Winter => 0.10,
    };
    let humidity_bias = (humidity - 0.5) * 0.4;
    (seasonal + humidity_bias).clamp(0.03, 0.85) as f32
}

/// 推进一个游戏小时粒度的天气状态转移（纯函数，可白盒测试）。
///
/// 雨天以固定概率转晴；晴天以下雨概率转为雨天；雾以独立概率生成并随时间消散。
pub fn step_weather(
    weather: &mut WeatherCell,
    rng: &mut impl RandomSource,
    season: Season,
    humidity: f64,
) {
    if weather.precipitation > 0.3 {
        // 雨天 → 概率转晴
        if rng.next_f32() < 0.45 {
            weather.precipitation = 0.0;
            weather.cloud_water = 0.2 + rng.next_f32() * 0.2;
        }
    } else {
        // 晴天 → 概率下雨（季节 + 湿度做主因）
        if rng.next_f32() < rain_probability(season, humidity) {
            weather.precipitation = 0.5 + rng.next_f32() * 0.5;
            weather.cloud_water = 0.65 + rng.next_f32() * 0.3;
        } else {
            weather.cloud_water = 0.15 + rng.next_f32() * 0.35;
        }
    }

    // 雾：独立概率生成，未触发时逐渐消散
    let fog_chance = 0.05 + humidity as f32 * 0.10;
    weather.fog_density = if rng.next_f32() < fog_chance {
        0.35 + rng.next_f32() * 0.5
    } else {
        (weather.fog_density * 0.7).max(0.0)
    };
}

/// 固定步天气系统：每 tick 重算派生温度；每个游戏小时边界按概率转移天气。
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

    // 温度每 tick 重算（平滑跟随昼夜与天气）。
    weather.temperature_c = compute_temperature_c(base_temperature, weather.precipitation, hour);

    // 天气维度只在跨小时边界时转移一次，避免同一小时内反复掷骰。
    if hour_events.read().count() > 0 {
        let game_hour =
            clock.total_game_minutes() / crate::game::world::time::MINUTES_PER_GAME_HOUR;
        let mut hour_rng = rng.for_event(WEATHER_RNG_DOMAIN, game_hour, 0);
        step_weather(&mut weather, &mut hour_rng, season, base_humidity);
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/weather/mod.rs"]
mod tests;
