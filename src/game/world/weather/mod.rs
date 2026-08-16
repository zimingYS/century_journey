//! 世界天气状态（Game 层权威，全局单 cell）。
//!
//! 天气与静态气候带（`ClimateSampler`）分层：气候带是种子决定的慢尺度外部条件
//! （基础温度/湿度 + 昼夜 + 季节），天气是快尺度的内部循环演化。
//!
//! 温度、湿度、云量三者互相耦合构成负反馈循环（而非单向因果）：
//! - 温度 → 湿度：高温蒸发增湿、低温凝露减湿；
//! - 湿度 → 云量：湿度越高越易成云；
//! - 云量 → 温度：云层遮蔽阳光降温。
//!
//! 降水是云量的产物，既降温又消耗水汽（凝结落地），进一步闭合循环。
//! 循环为负反馈，故在气候带约束下自稳定、不发散。

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
/// 云量、降水、雾霾是马尔可夫状态（存档）；温度与湿度是派生值（运行时计算、不存档）。
#[derive(Resource, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeatherCell {
    /// 云含水量 0~1（云系统消费）。
    pub cloud_water: f32,
    /// 降水强度 0~1（雨/雪/冰雹表现）。
    pub precipitation: f32,
    /// 雾霾密度 0~1。
    pub fog_density: f32,
    /// 当前温度（°C），由气候、昼夜、云遮蔽、降水派生；不存档。
    #[serde(skip)]
    pub temperature_c: f32,
    /// 当前湿度 0~1，由气候、蒸发、降水派生；不存档。
    #[serde(skip)]
    pub humidity: f32,
}

impl Default for WeatherCell {
    fn default() -> Self {
        Self {
            cloud_water: 0.3,
            precipitation: 0.0,
            fog_density: 0.0,
            temperature_c: 20.0,
            humidity: 0.5,
        }
    }
}

/// 昼夜温度修正因子，范围约 -0.87~0.87：14 点最热、2 点最冷。
pub fn diurnal_temperature_factor(hour: u8) -> f32 {
    let t = (hour as f32 - 6.0) / 24.0 * std::f32::consts::TAU;
    t.sin()
}

/// 由气候基础温度（0~1）、云量、降水与小时派生温度（°C）。
///
/// 气候 0~1 映射到 -10°C~50°C；昼夜修正 ±6°C；云层遮蔽约降 8°C；
/// 强降水再降 4°C。云遮蔽是「云量 → 温度」循环边的实现。
pub fn compute_temperature_c(
    base_temperature: f64,
    cloud_water: f32,
    precipitation: f32,
    hour: u8,
) -> f32 {
    let base = base_temperature as f32;
    let diurnal = diurnal_temperature_factor(hour);
    let cloud_cooling = -cloud_water * 8.0;
    let rain_cooling = -precipitation * 4.0;
    base * 60.0 - 10.0 + diurnal * 6.0 + cloud_cooling + rain_cooling
}

/// 由气候基础湿度（0~1）、温度与降水派生湿度（0~1）。
///
/// 蒸发是「温度 → 湿度」循环边：温度越高蒸发越强（基准 15°C）；降水消耗
/// 空气中的水汽（凝结落地），故降水会降低湿度。
pub fn compute_humidity(base_humidity: f64, temperature_c: f32, precipitation: f32) -> f32 {
    let base = base_humidity as f32;
    let evaporation = ((temperature_c - 15.0) * 0.01).clamp(-0.2, 0.3);
    (base + evaporation - precipitation * 0.15).clamp(0.0, 1.0)
}

/// 季节对成云的对流倾向：夏季对流强更易成云，冬季对流弱更易晴。
fn convective_bias(season: Season) -> f32 {
    match season {
        Season::Summer => 0.15,
        Season::Winter => -0.10,
        _ => 0.0,
    }
}

/// 推进一个游戏小时粒度的天气状态转移（纯函数，可白盒测试）。
///
/// 云量向「湿度 + 季节对流倾向」决定的目标漂移（「湿度 → 云量」循环边）；
/// 云量越过阈值后按概率产生降水；雾由湿度独立驱动并随时间消散。
pub fn step_weather(weather: &mut WeatherCell, rng: &mut impl RandomSource, season: Season) {
    let target_cloud = (weather.humidity + convective_bias(season)).clamp(0.0, 1.0);
    let drift = (target_cloud - weather.cloud_water) * 0.25;
    let jitter = (rng.next_f32() - 0.5) * 0.1;
    weather.cloud_water = (weather.cloud_water + drift + jitter).clamp(0.0, 1.0);

    // 降水：云量越过阈值后按云量线性增加概率，否则逐渐消散。
    if weather.cloud_water > 0.45 {
        let chance = (weather.cloud_water - 0.45) / 0.55 * 0.7;
        if rng.next_f32() < chance {
            weather.precipitation = 0.3 + rng.next_f32() * 0.6;
        } else {
            weather.precipitation *= 0.8;
        }
    } else {
        weather.precipitation *= 0.8;
    }

    // 雾：湿度驱动的独立概率，未触发时逐渐消散。
    let fog_chance = 0.03 + weather.humidity * 0.12;
    weather.fog_density = if rng.next_f32() < fog_chance {
        0.35 + rng.next_f32() * 0.5
    } else {
        (weather.fog_density * 0.7).max(0.0)
    };
}

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

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/weather/mod.rs"]
mod tests;
