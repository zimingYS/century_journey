use super::*;
use crate::game::world::block_ops::set_voxel_at_world;
use crate::game::world::chunk::ChunkData;
use crate::game::world::state::ChunkRuntime;
use crate::game::world::structure::TreeBlueprintParameters;
use crate::game::world::voxel_change::apply::apply_changes;
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
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer::default();

    let ok = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &world,      // &mut → &
        &mut buffer, // 新增
        |_| true,
    );
    assert!(ok);
    apply_changes(&mut world, &mut runtime, &mut buffer);

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
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer::default();

    assert!(try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &world,
        &mut buffer,
        |_| true,
    ));
    apply_changes(&mut world, &mut runtime, &mut buffer); // 先让 young 生效

    let young_positions = young
        .voxels()
        .iter()
        .map(|v| v.world_pos)
        .collect::<HashSet<_>>();
    let blocked_position = mature
        .voxels()
        .iter()
        .map(|v| v.world_pos)
        .find(|p| !young_positions.contains(p))
        .unwrap();
    set_voxel_at_world(blocked_position, 12, &mut world).unwrap();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Blueprint(&young),
        &mature,
        &tags,
        &world,
        &mut buffer,
        |_| true,
    );
    assert!(!result); // is_none → !result
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
    let mut runtime = ChunkRuntime::default();
    let mut buffer = VoxelChangeBuffer::default();

    assert!(try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young,
        &tags,
        &world,
        &mut buffer,
        |_| true,
    ));
    apply_changes(&mut world, &mut runtime, &mut buffer);

    let mature_positions = mature
        .voxels()
        .iter()
        .map(|v| v.world_pos)
        .collect::<HashSet<_>>();
    let old_only = young
        .voxels()
        .iter()
        .find(|v| !mature_positions.contains(&v.world_pos))
        .copied()
        .unwrap();

    assert!(try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Blueprint(&young),
        &mature,
        &tags,
        &world,
        &mut buffer,
        |_| true,
    ));
    apply_changes(&mut world, &mut runtime, &mut buffer);

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
    let mut buffer = VoxelChangeBuffer::default();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young_tree(root),
        &tags,
        &world,
        &mut buffer,
        |_| true,
    );
    assert!(!result);
    assert!(buffer.0.is_empty(), "预检失败不应提交命令");
    assert_eq!(get_voxel_at_world(root, &world), SAPLING_ID);
}

#[test]
fn missing_support_keeps_the_sapling() {
    let root = IVec3::new(8, 1, 8);
    let mut world = loaded_world(&[IVec3::ZERO]);
    set_voxel_at_world(root, SAPLING_ID, &mut world).unwrap();
    let (support_tag, tags) = support_tag_registry();
    let mut buffer = VoxelChangeBuffer::default();

    let result = try_apply_stage_transition(
        root,
        &support_tag,
        CurrentTreeForm::Sapling(SAPLING_ID),
        &young_tree(root),
        &tags,
        &world,
        &mut buffer,
        |_| true,
    );

    assert!(!result);
    assert_eq!(get_voxel_at_world(root, &world), SAPLING_ID);
}
