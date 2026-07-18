use crate::game::world::time::Season;
use noise::{NoiseFn, Perlin, Seedable};
use serde::{Deserialize, Serialize};

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

impl Season {
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
}

impl ClimateSampler {
    pub fn new(seed: u32, config: ClimateConfig) -> Self {
        Self {
            seed,
            temperature_noise: Perlin::new(seed).set_seed(seed),
            humidity_noise: Perlin::new(seed).set_seed(seed.wrapping_add(1000)),
            config,
        }
    }

    fn sample_base_temperature(&self, world_x: i32, world_z: i32) -> f64 {
        let raw = self.temperature_noise.get([
            world_x as f64 * self.config.temperature_scale,
            world_z as f64 * self.config.temperature_scale,
        ]);
        ((raw + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    fn sample_base_humidity(&self, world_x: i32, world_z: i32) -> f64 {
        let raw = self.humidity_noise.get([
            world_x as f64 * self.config.humidity_scale,
            world_z as f64 * self.config.humidity_scale,
        ]);
        ((raw + 1.0) * 0.5).clamp(0.0, 1.0)
    }

    /// 采样某点的温度 (0.0=极寒, 1.0=极热)
    pub fn sample_temperature_with_season(
        &self,
        world_x: i32,
        world_z: i32,
        season: Season,
    ) -> f64 {
        let base = self.sample_base_temperature(world_x, world_z);
        // 叠加季节偏移
        let seasonal = season.temperature_offset() * self.config.season_temperature_amplitude;
        (base + seasonal).clamp(0.0, 1.0)
    }

    /// 采样某点的湿度 (0.0=极干, 1.0=极湿)
    pub fn sample_humidity_with_season(&self, world_x: i32, world_z: i32, season: Season) -> f64 {
        let base = self.sample_base_humidity(world_x, world_z);
        let seasonal = season.humidity_offset() * self.config.season_humidity_amplitude;
        (base + seasonal).clamp(0.0, 1.0)
    }

    pub fn sample_temperature(&self, world_x: i32, world_z: i32) -> f64 {
        self.sample_base_temperature(world_x, world_z)
    }

    pub fn sample_humidity(&self, world_x: i32, world_z: i32) -> f64 {
        self.sample_base_humidity(world_x, world_z)
    }

    /// 基础生成气候只受生成版本控制，实时季节只能用于生成后的环境表现。
    pub fn sample_generation_temperature(
        &self,
        world_x: i32,
        world_z: i32,
        generation_version: u32,
    ) -> f64 {
        if generation_version == 1 {
            self.sample_temperature_with_season(world_x, world_z, Season::Spring)
        } else {
            self.sample_temperature(world_x, world_z)
        }
    }

    pub fn sample_generation_humidity(
        &self,
        world_x: i32,
        world_z: i32,
        generation_version: u32,
    ) -> f64 {
        if generation_version == 1 {
            self.sample_humidity_with_season(world_x, world_z, Season::Spring)
        } else {
            self.sample_humidity(world_x, world_z)
        }
    }
}

impl Clone for ClimateSampler {
    fn clone(&self) -> Self {
        Self::new(self.seed, self.config.clone())
    }
}
