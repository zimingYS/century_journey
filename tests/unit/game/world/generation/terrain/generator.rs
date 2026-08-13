use super::*;
use crate::content::biome::{BiomeDefinition, BiomeRegistry, BiomeTerrainParams};
use crate::game::world::generation::terrain::SEA_LEVEL;
use crate::game::world::generation::terrain::climate::ClimateConfig;
use crate::shared::identifier::Identifier;
use bevy::prelude::IVec3;

fn test_biomes() -> BiomeRegistry {
    BiomeRegistry::from_definitions(vec![BiomeDefinition {
        identifier: Identifier::parse("test:plains").unwrap(),
        display_name: "测试平原".into(),
        generation_order: 0,
        temperature_range: (0.0, 1.0),
        humidity_range: (0.0, 1.0),
        terrain: BiomeTerrainParams {
            base_height: 70.0,
            height_amplitude: 18.0,
            roughness: 0.6,
        },
        surface_block: Identifier::parse("test:grass").unwrap(),
        subsurface_block: Identifier::parse("test:dirt").unwrap(),
        beach_block: Identifier::parse("test:sand").unwrap(),
        tree_density: 0.0,
        ore_config: "test:ore".into(),
    }])
    .unwrap()
}

#[test]
fn direct_surface_sample_matches_chunk_context_columns() {
    let seed = 0xC317_2026;
    let key = BaseGenerationKey {
        seed,
        chunk_pos: IVec3::new(-3, 0, 5),
        generation_version: 2,
    };
    let noise = NoiseSampler::new(seed);
    let climate = ClimateSampler::new(seed, ClimateConfig::default());
    let biomes = test_biomes();
    let context = TerrainGenerator::sample_context(&noise, &climate, &biomes, key);

    for (local_x, local_z) in [(0, 0), (7, 11), (15, 15)] {
        let column = context.get_column(local_x, local_z);
        let sample = TerrainGenerator::sample_surface(
            &noise,
            &climate,
            &biomes,
            key,
            column.world_x,
            column.world_z,
        );

        assert_eq!(sample.ground_height, column.base_height);
        assert_eq!(sample.biome_index, column.biome_index);
        assert_eq!(sample.temperature, column.temperature);
        assert_eq!(sample.humidity, column.humidity);
        let biome = biomes.get(column.biome_index).unwrap();
        let expected_surface = if column.base_height <= SEA_LEVEL + 2 {
            &biome.beach_block
        } else {
            &biome.surface_block
        };
        let expected_subsurface = if column.base_height <= SEA_LEVEL {
            &biome.beach_block
        } else {
            &biome.subsurface_block
        };
        assert_eq!(&sample.surface_block, expected_surface);
        assert_eq!(&sample.subsurface_block, expected_subsurface);
    }
}

#[test]
fn surface_sample_is_deterministic_for_seed_coordinate_and_generation_version() {
    let seed = 0xC317_2026;
    let key = BaseGenerationKey {
        seed,
        chunk_pos: IVec3::ZERO,
        generation_version: crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION,
    };
    let noise = NoiseSampler::new(seed);
    let climate = ClimateSampler::new(seed, ClimateConfig::default());
    let biomes = test_biomes();

    let first = TerrainGenerator::sample_surface(&noise, &climate, &biomes, key, -129, 257);
    let second = TerrainGenerator::sample_surface(&noise, &climate, &biomes, key, -129, 257);

    assert_eq!(first, second);
}

#[test]
fn surface_sample_applies_the_same_sea_level_surface_contract() {
    let seed = 0xC317_2026;
    let biomes = BiomeRegistry::from_definitions(vec![BiomeDefinition {
        identifier: Identifier::parse("test:deep_water").unwrap(),
        display_name: "测试深水".into(),
        generation_order: 0,
        temperature_range: (0.0, 1.0),
        humidity_range: (0.0, 1.0),
        terrain: BiomeTerrainParams {
            base_height: 0.0,
            height_amplitude: 0.0,
            roughness: 0.0,
        },
        surface_block: Identifier::parse("test:grass").unwrap(),
        subsurface_block: Identifier::parse("test:dirt").unwrap(),
        beach_block: Identifier::parse("test:sand").unwrap(),
        tree_density: 0.0,
        ore_config: "test:ore".into(),
    }])
    .unwrap();
    let noise = NoiseSampler::new(seed);
    let climate = ClimateSampler::new(seed, ClimateConfig::default());
    let key = BaseGenerationKey {
        seed,
        chunk_pos: IVec3::ZERO,
        generation_version: crate::game::world::generation::pipeline::CURRENT_GENERATION_VERSION,
    };

    let sample = TerrainGenerator::sample_surface(&noise, &climate, &biomes, key, 11, -7);

    assert_eq!(sample.ground_height, 0);
    assert_eq!(sample.visible_surface_height, SEA_LEVEL + 1);
    assert!(sample.is_water_surface);
    assert_eq!(
        sample.surface_block,
        Identifier::parse("test:sand").unwrap()
    );
    assert_eq!(
        sample.subsurface_block,
        Identifier::parse("test:sand").unwrap()
    );
}
