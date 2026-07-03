use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 生物群系定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDefinition {
    pub identifier: String,
    pub display_name: String,
    pub temperature_range: (f64, f64),
    pub humidity_range: (f64, f64),
    pub terrain: BiomeTerrainParams,
    pub surface_block: String,
    pub subsurface_block: String,
    pub beach_block: String,
    pub tree_density: f32,
    pub ore_config: String,
}

/// 生物群系的地形参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeTerrainParams {
    pub base_height: f64,
    pub height_amplitude: f64,
    pub roughness: f64,
}

impl Default for BiomeTerrainParams {
    fn default() -> Self {
        Self {
            base_height: 64.0,
            height_amplitude: 20.0,
            roughness: 0.5,
        }
    }
}
