use super::*;
use crate::game::world::chunk::ChunkData;
use crate::shared::identifier::Identifier;
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

fn prepare_sapling(world: &mut WorldState, anchor: IVec3) {
    set_voxel_at_world(anchor - IVec3::Y, SUPPORT_ID, world).unwrap();
    set_voxel_at_world(anchor, SAPLING_ID, world).unwrap();
}

fn small_tree(anchor: IVec3) -> TreeBlueprint {
    TreeBlueprint::generate(
        anchor,
        0,
        TRUNK_ID,
        LEAVES_ID,
        TreeBlueprintParameters::generated_tree(),
    )
}

fn tree_instance(anchor: IVec3) -> TreeInstance {
    TreeInstance::new_mature(anchor, Identifier::new("century_journey", "oak"), 0, 25)
}

#[test]
fn valid_sapling_replaces_itself_with_a_complete_tree() {
    let anchor = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, anchor);
    let (support_tag, tags) = support_tag_registry();
    let blueprint = small_tree(anchor);

    let changes = try_apply_tree_growth(
        SAPLING_ID,
        &support_tag,
        &blueprint,
        tree_instance(anchor),
        &tags,
        &mut world,
        |_| true,
    )
    .unwrap();

    assert_eq!(changes.len(), blueprint.voxels().len());
    assert_eq!(get_voxel_at_world(anchor, &world), TRUNK_ID);
    for voxel in blueprint.voxels() {
        assert_eq!(get_voxel_at_world(voxel.world_pos, &world), voxel.block_id);
    }
    let instance = world.tree_instance(anchor).unwrap();
    assert_eq!(instance.root(), anchor);
    assert_eq!(instance.shape_seed(), 0);
}

#[test]
fn unsupported_sapling_keeps_the_world_unchanged() {
    let anchor = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    set_voxel_at_world(anchor, SAPLING_ID, &mut world).unwrap();
    let (support_tag, tags) = support_tag_registry();

    let result = try_apply_tree_growth(
        SAPLING_ID,
        &support_tag,
        &small_tree(anchor),
        tree_instance(anchor),
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(anchor, &world), SAPLING_ID);
    assert!(world.tree_instance(anchor).is_none());
}

#[test]
fn blocked_tree_space_produces_no_partial_writes() {
    let anchor = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, anchor);
    let blocked_position = anchor + IVec3::Y;
    set_voxel_at_world(blocked_position, 12, &mut world).unwrap();
    let (support_tag, tags) = support_tag_registry();

    let result = try_apply_tree_growth(
        SAPLING_ID,
        &support_tag,
        &small_tree(anchor),
        tree_instance(anchor),
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(anchor, &world), SAPLING_ID);
    assert_eq!(get_voxel_at_world(blocked_position, &world), 12);
}

#[test]
fn blueprint_crossing_an_unloaded_chunk_keeps_the_sapling() {
    let anchor = IVec3::new(15, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    prepare_sapling(&mut world, anchor);
    let (support_tag, tags) = support_tag_registry();

    let result = try_apply_tree_growth(
        SAPLING_ID,
        &support_tag,
        &small_tree(anchor),
        tree_instance(anchor),
        &tags,
        &mut world,
        |_| true,
    );

    assert!(result.is_none());
    assert_eq!(get_voxel_at_world(anchor, &world), SAPLING_ID);
    assert!(world.tree_instance(anchor).is_none());
}

#[test]
fn growth_minute_phase_is_stable_for_the_same_position() {
    let position = IVec3::new(-4, 70, 19);
    let due_minutes = (0..20)
        .filter(|minute| growth_attempt_is_due(position, SAPLING_ID, *minute, 5))
        .collect::<Vec<_>>();

    assert_eq!(due_minutes.len(), 4);
    assert!(due_minutes.windows(2).all(|pair| pair[1] - pair[0] == 5));
}
