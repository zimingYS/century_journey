use crate::content::biome::definition::{BiomeDefinition, BiomeTerrainParams};
use bevy::prelude::*;
use std::collections::HashMap;

/// 生物群系注册表
#[derive(Resource, Default, Clone)]
pub struct BiomeRegistry {
    pub biomes: Vec<BiomeDefinition>,
    pub identifier_to_index: HashMap<String, u8>,
}

impl BiomeRegistry {
    pub fn register_builtin_biomes(&mut self) {
        let builtin = vec![
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
            self.identifier_to_index
                .insert(biome.identifier.clone(), idx);
            self.biomes.push(biome);
        }
    }

    pub fn get(&self, index: u8) -> Option<&BiomeDefinition> {
        self.biomes.get(index as usize)
    }

    pub fn get_index(&self, identifier: &str) -> Option<u8> {
        self.identifier_to_index.get(identifier).copied()
    }
}
