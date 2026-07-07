use bevy::prelude::IVec3;
use century_journey::game::world::generation::WorldGenerator;
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

#[test]
fn world_generation_is_deterministic_for_same_seed_and_chunk() {
    let chunk_pos = IVec3::new(3, 4, -5);
    let block_ids = test_block_ids();
    let first = WorldGenerator::new(42).generate_chunk_data(chunk_pos, block_ids.clone());
    let second = WorldGenerator::new(42).generate_chunk_data(chunk_pos, block_ids);

    assert_eq!(first.voxels, second.voxels);
}

#[test]
fn world_generation_changes_for_different_seeds() {
    let chunk_pos = IVec3::new(3, 4, -5);
    let block_ids = test_block_ids();
    let first = WorldGenerator::new(42).generate_chunk_data(chunk_pos, block_ids.clone());
    let second = WorldGenerator::new(43).generate_chunk_data(chunk_pos, block_ids);

    assert_ne!(first.voxels, second.voxels);
}
