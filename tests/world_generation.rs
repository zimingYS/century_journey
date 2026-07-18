use bevy::prelude::IVec3;
use century_journey::content::biome::{BiomeDefinition, BiomeRegistry};
use century_journey::content::format::load_versioned_json_dir;
use century_journey::engine::asset::{AssetFiles, AssetResolver};
use century_journey::game::world::generation::WorldGenerator;
use century_journey::game::world::generation::climate::Season;
use century_journey::game::world::generation::noise::GenerationBlockIds;
use std::collections::HashSet;

fn test_block_ids() -> GenerationBlockIds {
    GenerationBlockIds {
        air: 0,
        grass: 1,
        dirt: 2,
        stone: 3,
        sand: 4,
        water: 5,
        snow: 6,
        leaves: 7,
        wood: 8,
        tree_plantable_ids: HashSet::from([1, 2]),
        natural_ids: HashSet::from([1, 2, 3, 4, 6, 7, 8]),
        overworld_replaceable_ids: HashSet::from([0, 5]),
    }
}

fn test_biomes() -> BiomeRegistry {
    let resolver =
        AssetResolver::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    let files = AssetFiles::new(&resolver);
    let definitions = load_versioned_json_dir::<BiomeDefinition>(&files, "definitions/biomes")
        .into_iter()
        .map(|(_, biome)| biome)
        .collect();
    BiomeRegistry::from_definitions(definitions).unwrap()
}

#[test]
fn world_generation_is_deterministic_for_same_seed_and_chunk() {
    let chunk_pos = IVec3::new(3, 4, -5);
    let block_ids = test_block_ids();
    let first =
        WorldGenerator::new(42, test_biomes()).generate_chunk_data(chunk_pos, block_ids.clone());
    let second = WorldGenerator::new(42, test_biomes()).generate_chunk_data(chunk_pos, block_ids);

    assert_eq!(first.voxels, second.voxels);
}

#[test]
fn world_generation_changes_for_different_seeds() {
    let chunk_pos = IVec3::new(3, 4, -5);
    let block_ids = test_block_ids();
    let first =
        WorldGenerator::new(42, test_biomes()).generate_chunk_data(chunk_pos, block_ids.clone());
    let second = WorldGenerator::new(43, test_biomes()).generate_chunk_data(chunk_pos, block_ids);

    assert_ne!(first.voxels, second.voxels);
}

#[test]
fn base_voxels_and_biomes_are_identical_in_every_season() {
    let chunk_pos = IVec3::new(-7, 3, 11);
    let block_ids = test_block_ids();
    let generator = WorldGenerator::new(42, test_biomes());
    let baseline_chunk = generator.generate_chunk_data(chunk_pos, block_ids.clone());
    let baseline_biomes = generator
        .pipeline
        .sample_context(chunk_pos)
        .columns
        .into_iter()
        .map(|column| column.biome_index)
        .collect::<Vec<_>>();

    for season in [
        Season::Spring,
        Season::Summer,
        Season::Autumn,
        Season::Winter,
    ] {
        // 季节气候仍可供环境表现使用，但它不进入基础生成 API。
        let _environment_temperature = generator
            .pipeline
            .climate_sampler
            .sample_temperature_with_season(0, 0, season);
        let generated = generator.generate_chunk_data(chunk_pos, block_ids.clone());
        let generated_biomes = generator
            .pipeline
            .sample_context(chunk_pos)
            .columns
            .into_iter()
            .map(|column| column.biome_index)
            .collect::<Vec<_>>();

        assert_eq!(generated.voxels, baseline_chunk.voxels, "season={season:?}");
        assert_eq!(generated_biomes, baseline_biomes, "season={season:?}");
    }
}
