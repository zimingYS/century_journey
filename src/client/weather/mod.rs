//! 降水粒子表现：读权威天气的降水强度与温度，环绕玩家渲染雨滴或雪花。
//!
//! 纯表现层：只把 `WeatherCell.precipitation` 与 `temperature_c` 转成可见粒子，
//! 不参与任何权威规则。粒子数量随降水强度增减，温度低于冰点时下雪、否则下雨。

mod plugin;
mod systems;
mod types;

pub use plugin::ClientWeatherPlugin;

#[cfg(test)]
#[path = "../../../tests/unit/client/weather/mod.rs"]
mod tests;
