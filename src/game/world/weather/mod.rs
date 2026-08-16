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

mod cell;
mod evolution;
mod plugin;
mod systems;

pub use cell::WeatherCell;
pub use evolution::{
    compute_humidity, compute_temperature_c, diurnal_temperature_factor, step_weather,
};
pub use plugin::WeatherPlugin;
pub use systems::weather_evolve_system;

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/weather/mod.rs"]
mod tests;
