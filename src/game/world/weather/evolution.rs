//! 天气演化的纯函数逻辑（温度/湿度派生与马尔可夫状态转移，可白盒测试）。

use crate::game::world::time::Season;
use crate::game::world::weather::cell::WeatherCell;
use crate::shared::random::RandomSource;

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
