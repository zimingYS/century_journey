use super::*;
use crate::game::world::chunk::ChunkData;
use crate::game::world::structure::TreeBlueprintParameters;
use std::collections::HashSet;
use std::sync::Arc;

const SAPLING_ID: u16 = 5;
const SUPPORT_ID: u16 = 7;
const TRUNK_ID: u16 = 9;
const LEAVES_ID: u16 = 10;

fn support_tag_registry() -> (TagId, RuntimeTagRegistry) {
    let tag = TagId::new("test", "tree_supports");
    let mut ids = HashSet::new();
    ids.insert(SUPPORT_ID);
    let mut registry = RuntimeTagRegistry::default();
    registry.insert(tag.clone(), ids);
    (tag, registry)
}

fn loaded_world(chunk_positions: &[IVec3]) -> WorldState {
    let mut world = WorldState::default();
    for &position in chunk_positions {
        world.insert_chunk(position, Arc::new(ChunkData::new()));
    }
    world
}

fn prepare_sapling(world: &mut WorldState, root: IVec3) {
    set_voxel_at_world(root - IVec3::Y, SUPPORT_ID, world).unwrap();
    set_voxel_at_world(root, SAPLING_ID, world).unwrap();
}

fn young_tree(root: IVec3) -> TreeBlueprint {
    TreeBlueprint::generate(
        root,
        0,
        TRUNK_ID,
        LEAVES_ID,
        TreeBlueprintParameters {
            trunk_height_min: 2,
            trunk_height_max: 2,
            crown_radius_min: 1,
            crown_radius_max: 1,
        },
    )
}

fn mature_tree(root: IVec3) -> TreeBlueprint {
    TreeBlueprint::generate(
        root,
        0,
        TRUNK_ID,
        LEAVES_ID,
        TreeBlueprintParameters {
            trunk_height_min: 4,
            trunk_height_max: 4,
            crown_radius_min: 2,
            crown_radius_max: 2,
        },
    )
}

#[test]
fn valid_sapling_is_atomically_replaced_by_the_young_blueprint() {
    let root = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, root);
    let (support_tag, tags) = support_tag_registry();
    let young = young_tree(root);

    let changes = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &mut world,
        |_| true,
    )
    .unwrap();

    assert_eq!(changes.len(), young.voxels().len());
    for voxel in young.voxels() {
        assert_eq!(get_voxel_at_world(voxel.world_pos, &world), voxel.block_id);
    }
}

#[test]
fn blocked_new_space_keeps_the_entire_young_tree_unchanged() {
    let root = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, root);
    let (support_tag, tags) = support_tag_registry();
    let young = young_tree(root);
    let mature = mature_tree(root);
    try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &mut world,
        |_| true,
    )
    .unwrap();

    let young_positions = young
        .voxels()
        .iter()
        .map(|voxel| voxel.world_pos)
        .collect::<HashSet<_>>();
    let blocked_position = mature
        .voxels()
        .iter()
        .map(|voxel| voxel.world_pos)
        .find(|position| !young_positions.contains(position))
        .unwrap();
    set_voxel_at_world(blocked_position, 12, &mut world).unwrap();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Blueprint(&young),
        &mature,
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(blocked_position, &world), 12);
    for voxel in young.voxels() {
        assert_eq!(get_voxel_at_world(voxel.world_pos, &world), voxel.block_id);
    }
}

#[test]
fn young_to_mature_keeps_previous_branches_and_adds_the_mature_blueprint() {
    let root = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, root);
    let (support_tag, tags) = support_tag_registry();
    let young = young_tree(root);
    let mature = mature_tree(root);
    try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &mut world,
        |_| true,
    )
    .unwrap();
    let mature_positions = mature
        .voxels()
        .iter()
        .map(|voxel| voxel.world_pos)
        .collect::<HashSet<_>>();
    let old_only = young
        .voxels()
        .iter()
        .find(|voxel| !mature_positions.contains(&voxel.world_pos))
        .copied()
        .unwrap();

    try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Blueprint(&young),
        &mature,
        &tags,
        &mut world,
        |_| true,
    )
    .unwrap();

    assert_eq!(
        get_voxel_at_world(old_only.world_pos, &world),
        old_only.block_id
    );
    for voxel in mature.voxels() {
        assert_eq!(get_voxel_at_world(voxel.world_pos, &world), voxel.block_id);
    }
}

#[test]
fn unloaded_target_chunk_keeps_the_sapling() {
    let root = IVec3::new(15, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, root);
    let (support_tag, tags) = support_tag_registry();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young_tree(root),
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(root, &world), SAPLING_ID);
}

#[test]
fn missing_support_keeps_the_sapling() {
    let root = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    set_voxel_at_world(root, SAPLING_ID, &mut world).unwrap();
    let (support_tag, tags) = support_tag_registry();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young_tree(root),
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(root, &world), SAPLING_ID);
}
