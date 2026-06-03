use std::collections::HashMap;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// 生物群系定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeDefinition {
    /// 唯一标识符
    pub identifier: String,
    /// 显示名称
    pub display_name: String,
    /// 温度范围 (min, max)，0.0~1.0
    pub temperature_range: (f64, f64),
    /// 湿度范围 (min, max)，0.0~1.0
    pub humidity_range: (f64, f64),
    /// 地形参数
    pub terrain: BiomeTerrainParams,
    /// 地表方块标识符
    pub surface_block: String,
    /// 地表下方块标识符（泥土层）
    pub subsurface_block: String,
    /// 水边方块标识符（沙滩）
    pub beach_block: String,
    /// 树木生成密度 (0.0=无, 1.0=密集)
    pub tree_density: f32,
    /// 矿脉生成配置标识符
    pub ore_config: String,
}

/// 生物群系的地形参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeTerrainParams {
    /// 基础高度偏移
    pub base_height: f64,
    /// 高度振幅（越大山越高）
    pub height_amplitude: f64,
    /// 粗糙度（越大越崎岖）
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

/// 生物群系注册表
#[derive(Resource, Default)]
pub struct BiomeRegistry {
    /// 所有生物群系定义
    pub biomes: Vec<BiomeDefinition>,
    /// 标识符 → 索引
    pub identifier_to_index: HashMap<String, u8>,
}

impl BiomeRegistry {
    /// 注册内置生物群系
    pub fn register_builtin_biomes(&mut self) {
        let builtin = vec![
            // 平原
            BiomeDefinition {
                identifier: "century_journey:plains".to_string(),
                display_name: "平原".to_string(),
                temperature_range: (0.3, 0.7),
                humidity_range: (0.3, 0.7),
                terrain: BiomeTerrainParams {
                    base_height: 64.0,
                    height_amplitude: 8.0,
                    roughness: 0.2,
                },
                surface_block: "century_journey:grass".to_string(),
                subsurface_block: "century_journey:dirt".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.02,
                ore_config: "standard".to_string(),
            },
            // 森林
            BiomeDefinition {
                identifier: "century_journey:forest".to_string(),
                display_name: "森林".to_string(),
                temperature_range: (0.3, 0.7),
                humidity_range: (0.5, 1.0),
                terrain: BiomeTerrainParams {
                    base_height: 64.0,
                    height_amplitude: 12.0,
                    roughness: 0.4,
                },
                surface_block: "century_journey:grass".to_string(),
                subsurface_block: "century_journey:dirt".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.15,
                ore_config: "standard".to_string(),
            },
            // 沙漠
            BiomeDefinition {
                identifier: "century_journey:desert".to_string(),
                display_name: "沙漠".to_string(),
                temperature_range: (0.7, 1.0),
                humidity_range: (0.0, 0.3),
                terrain: BiomeTerrainParams {
                    base_height: 64.0,
                    height_amplitude: 6.0,
                    roughness: 0.15,
                },
                surface_block: "century_journey:sand".to_string(),
                subsurface_block: "century_journey:sand".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.0,
                ore_config: "desert".to_string(),
            },
            // 雪山
            BiomeDefinition {
                identifier: "century_journey:snowy_mountains".to_string(),
                display_name: "雪山".to_string(),
                temperature_range: (0.0, 0.25),
                humidity_range: (0.3, 0.8),
                terrain: BiomeTerrainParams {
                    base_height: 80.0,
                    height_amplitude: 40.0,
                    roughness: 0.7,
                },
                surface_block: "century_journey:grass".to_string(),
                subsurface_block: "century_journey:dirt".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.01,
                ore_config: "mountain".to_string(),
            },
            // 冻原
            BiomeDefinition {
                identifier: "century_journey:tundra".to_string(),
                display_name: "冻原".to_string(),
                temperature_range: (0.0, 0.25),
                humidity_range: (0.0, 0.4),
                terrain: BiomeTerrainParams {
                    base_height: 64.0,
                    height_amplitude: 5.0,
                    roughness: 0.1,
                },
                surface_block: "century_journey:grass".to_string(),
                subsurface_block: "century_journey:dirt".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.0,
                ore_config: "standard".to_string(),
            },
            // 海洋
            BiomeDefinition {
                identifier: "century_journey:ocean".to_string(),
                display_name: "海洋".to_string(),
                temperature_range: (0.2, 0.8),
                humidity_range: (0.6, 1.0),
                terrain: BiomeTerrainParams {
                    base_height: 50.0,
                    height_amplitude: 5.0,
                    roughness: 0.1,
                },
                surface_block: "century_journey:sand".to_string(),
                subsurface_block: "century_journey:sand".to_string(),
                beach_block: "century_journey:sand".to_string(),
                tree_density: 0.0,
                ore_config: "ocean".to_string(),
            },
        ];

        for biome in builtin {
            let idx = self.biomes.len() as u8;
            self.identifier_to_index.insert(biome.identifier.clone(), idx);
            self.biomes.push(biome);
        }
    }

    /// 根据温度/湿度选择生物群系
    pub fn select_biome(&self, temperature: f64, humidity: f64) -> u8 {
        let mut best_index = 0u8;
        let mut best_score = f64::MAX;

        for (idx, biome) in self.biomes.iter().enumerate() {
            let temp_center = (biome.temperature_range.0 + biome.temperature_range.1) * 0.5;
            let humid_center = (biome.humidity_range.0 + biome.humidity_range.1) * 0.5;

            let temp_dist = (temperature - temp_center).abs();
            let humid_dist = (humidity - humid_center).abs();

            // 加权距离：温度权重略高于湿度
            let score = temp_dist * 1.2 + humid_dist;

            if score < best_score {
                best_score = score;
                best_index = idx as u8;
            }
        }

        best_index
    }

    /// 混合所有群系的地形参数
    pub fn blend_terrain_params(&self, temperature: f64, humidity: f64) -> BiomeTerrainParams {
        let mut total_weight = 0.0;
        let mut blended = BiomeTerrainParams {
            base_height: 0.0,
            height_amplitude: 0.0,
            roughness: 0.0,
        };

        for biome in &self.biomes {
            let temp_center = (biome.temperature_range.0 + biome.temperature_range.1) * 0.5;
            let humid_center = (biome.humidity_range.0 + biome.humidity_range.1) * 0.5;
            let temp_dist = (temperature - temp_center).abs();
            let humid_dist = (humidity - humid_center).abs();
            let score = temp_dist * 1.2 + humid_dist;

            // 高斯权重：距离越远权重衰减越快
            let weight = (-score * score * 6.0).exp();

            blended.base_height += biome.terrain.base_height * weight;
            blended.height_amplitude += biome.terrain.height_amplitude * weight;
            blended.roughness += biome.terrain.roughness * weight;
            total_weight += weight;
        }

        if total_weight > 0.0 {
            blended.base_height /= total_weight;
            blended.height_amplitude /= total_weight;
            blended.roughness /= total_weight;
        }

        blended
    }

    /// 通过索引获取生物群系定义
    pub fn get(&self, index: u8) -> Option<&BiomeDefinition> {
        self.biomes.get(index as usize)
    }

    /// 通过标识符获取索引
    pub fn get_index(&self, identifier: &str) -> Option<u8> {
        self.identifier_to_index.get(identifier).copied()
    }
}