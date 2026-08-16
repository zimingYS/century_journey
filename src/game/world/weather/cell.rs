//! 全局天气状态单元及其派生字段定义。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

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
