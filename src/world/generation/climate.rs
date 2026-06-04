use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Seedable};
use serde::{Serialize, Deserialize};

/// 气候配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateConfig {
    /// 温度噪声
    pub temperature_scale: f64,
    /// 湿度噪声
    pub humidity_scale: f64,
    /// 季节影响温度的振幅
    pub season_temperature_amplitude: f64,
    /// 季节影响湿度的振幅
    pub season_humidity_amplitude: f64,
}

impl Default for ClimateConfig {
    fn default() -> Self {
        Self {
            temperature_scale: 0.001,
            humidity_scale: 0.0015,
            season_temperature_amplitude: 0.15,
            season_humidity_amplitude: 0.10,
        }
    }
}

/// 四季枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// 从世界时间获取当前季节
    pub fn from_time(time_of_day: f32) -> Self {
        let season_res = SeasonResource::default();
        season_res.current_season(time_of_day)
    }

    /// 季节对温度的偏移 (-1.0 ~ 1.0)
    pub fn temperature_offset(&self) -> f64 {
        match self {
            Season::Spring => 0.0,
            Season::Summer => 1.0,
            Season::Autumn => 0.0,
            Season::Winter => -1.0,
        }
    }

    /// 季节对湿度的偏移 (-1.0 ~ 1.0)
    pub fn humidity_offset(&self) -> f64 {
        match self {
            Season::Spring => 0.5,
            Season::Summer => -0.2,
            Season::Autumn => 0.0,
            Season::Winter => -0.3,
        }
    }
}

/// 气候采样器 — 从世界坐标采样温度/湿度
pub struct ClimateSampler {
    pub seed: u32,
    pub temperature_noise: Perlin,
    pub humidity_noise: Perlin,
    pub config: ClimateConfig,
    /// 当前季节（可动态更新）
    pub current_season: Season,
}

impl ClimateSampler {
    pub fn new(seed: u32, config: ClimateConfig) -> Self {
        Self {
            seed,
            temperature_noise: Perlin::new(seed).set_seed(seed),
            humidity_noise: Perlin::new(seed).set_seed(seed.wrapping_add(1000)),
            config,
            current_season: Season::Spring,
        }
    }

    /// 采样某点的温度 (0.0=极寒, 1.0=极热)
    pub fn sample_temperature(&self, world_x: i32, world_z: i32) -> f64 {
        let raw = self.temperature_noise.get([
            world_x as f64 * self.config.temperature_scale,
            world_z as f64 * self.config.temperature_scale,
        ]);
        // 原始 Perlin 输出大约 -1.0 ~ 1.0，映射到 0.0 ~ 1.0
        let base = (raw + 1.0) * 0.5;
        // 叠加季节偏移
        let seasonal = self.current_season.temperature_offset()
            * self.config.season_temperature_amplitude;
        (base + seasonal).clamp(0.0, 1.0)
    }

    /// 采样某点的湿度 (0.0=极干, 1.0=极湿)
    pub fn sample_humidity(&self, world_x: i32, world_z: i32) -> f64 {
        let raw = self.humidity_noise.get([
            world_x as f64 * self.config.humidity_scale,
            world_z as f64 * self.config.humidity_scale,
        ]);
        let base = (raw + 1.0) * 0.5;
        let seasonal = self.current_season.humidity_offset()
            * self.config.season_humidity_amplitude;
        (base + seasonal).clamp(0.0, 1.0)
    }
}

impl Clone for ClimateSampler {
    fn clone(&self) -> Self {
        let mut new = Self::new(self.seed, self.config.clone());
        new.current_season = self.current_season;
        new
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct SeasonResource {
    /// 一个季节持续多少游戏日
    /// （默认12天一个季度,对应2天一个节气）
    pub days_per_season: f32,
}

impl Default for SeasonResource {
    fn default() -> Self {
        Self {
            days_per_season: 12.0,
        }
    }
}

impl SeasonResource {
    /// 从世界时间（天数）计算当前季节
    pub fn current_season(&self, time_of_day: f32) -> Season {
        const SECONDS_PER_DAY: f32 = 1440.0;
        let total_days = time_of_day / SECONDS_PER_DAY;
        let day_in_year = total_days % (self.days_per_season * 4.0);
        let season_progress = day_in_year / self.days_per_season;

        match season_progress as u32 % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }
}