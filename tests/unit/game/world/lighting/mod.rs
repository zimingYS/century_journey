use super::WorldLighting;
use super::resources::{
    CachedLightInfo, LightingBuildResult, LightingRebuildTracker,
    WORLD_REBUILD_MAX_TASK_DEFER_TICKS, WORLD_REBUILD_STABLE_TICKS,
};
use super::systems::{
    changed_light_chunks, light_dependent_mesh_chunks, lighting_result_is_current,
};
use crate::game::world::chunk::ChunkData;
use crate::game::world::lighting::chunk_light::{ChunkLight, LightCell};
use crate::game::world::lighting::rebuild::LightingWorldSnapshot;
use crate::game::world::state::WorldState;
use bevy::math::IVec3;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn changed_light_chunks_only_returns_actual_differences() {
    let unchanged_pos = IVec3::ZERO;
    let changed_pos = IVec3::X;
    let removed_pos = IVec3::Y;
    let added_pos = IVec3::Z;

    let mut previous = HashMap::new();
    let mut rebuilt = HashMap::new();
    let mut unchanged = ChunkLight::default();
    unchanged.mark_initialized();
    previous.insert(unchanged_pos, Arc::new(unchanged.clone()));
    rebuilt.insert(unchanged_pos, Arc::new(unchanged));

    previous.insert(changed_pos, Arc::new(ChunkLight::default()));
    let mut changed = ChunkLight::default();
    changed.mark_initialized();
    rebuilt.insert(changed_pos, Arc::new(changed));
    previous.insert(removed_pos, Arc::new(ChunkLight::default()));
    rebuilt.insert(added_pos, Arc::new(ChunkLight::default()));

    let affected = changed_light_chunks(&previous, &rebuilt);

    assert!(!affected.contains(&unchanged_pos));
    assert!(affected.contains(&changed_pos));
    assert!(affected.contains(&removed_pos));
    assert!(affected.contains(&added_pos));
}

#[test]
fn changed_light_invalidates_neighbor_boundary_meshes() {
    let changed = HashSet::from([IVec3::new(3, 4, 5)]);
    let affected = light_dependent_mesh_chunks(&changed);

    assert_eq!(affected.len(), 7);
    assert!(affected.contains(&IVec3::new(4, 4, 5)));
    assert!(affected.contains(&IVec3::new(3, 3, 5)));
}

#[test]
fn world_light_lookup_handles_negative_chunk_coordinates() {
    let mut lighting = WorldLighting::default();
    let mut light = ChunkLight::default();
    light.set(
        15,
        2,
        3,
        LightCell {
            sky: Default::default(),
            block: crate::game::world::lighting::chunk_light::LightRgb { r: 9, g: 4, b: 1 },
        },
    );
    light.mark_initialized();
    lighting
        .chunk_lights
        .insert(IVec3::new(-1, 0, 0), Arc::new(light));

    let sampled = lighting.light_cell_at_world(IVec3::new(-1, 2, 3)).unwrap();
    assert_eq!(sampled.block.r, 9);
}

#[test]
fn current_light_requires_initialized_data_and_matching_snapshot() {
    let position = IVec3::ZERO;
    let data = Arc::new(ChunkData::new());
    let mut lighting = WorldLighting::default();
    lighting.chunk_snapshots.insert(position, Arc::clone(&data));
    lighting
        .chunk_lights
        .insert(position, Arc::new(ChunkLight::default()));
    assert!(!lighting.is_chunk_light_current(position, &data));

    let mut initialized = ChunkLight::default();
    initialized.mark_initialized();
    lighting
        .chunk_lights
        .insert(position, Arc::new(initialized));
    assert!(lighting.is_chunk_light_current(position, &data));
    assert!(!lighting.is_chunk_light_current(position, &Arc::new(ChunkData::new())));
}

#[test]
fn streaming_changes_are_coalesced_until_the_world_is_stable() {
    let revision = 1;
    let mut tracker = LightingRebuildTracker::default();

    tracker.observe(revision, 0);
    assert!(!tracker.ready_to_dispatch(0));
    for _ in 1..WORLD_REBUILD_STABLE_TICKS {
        tracker.observe(revision, 0);
        assert!(!tracker.ready_to_dispatch(0));
    }
    tracker.observe(revision, 0);

    assert!(tracker.ready_to_dispatch(0));
    assert!(!tracker.ready_to_dispatch(1));
}

#[test]
fn content_changes_bypass_the_streaming_stability_delay() {
    let mut tracker = LightingRebuildTracker::default();

    tracker.observe(0, 1);

    assert!(tracker.ready_to_dispatch(0));
}

#[test]
fn global_rebuild_task_backlog_has_a_bounded_deferral() {
    let mut tracker = LightingRebuildTracker {
        pending: true,
        stable_ticks: WORLD_REBUILD_STABLE_TICKS,
        ..Default::default()
    };
    for _ in 1..WORLD_REBUILD_MAX_TASK_DEFER_TICKS {
        assert!(tracker.should_defer_for_task_backlog(1));
    }

    assert!(!tracker.should_defer_for_task_backlog(1));
    assert!(!tracker.should_defer_for_task_backlog(0));
    assert_eq!(tracker.task_defer_ticks, 0);
}

#[test]
fn result_is_rejected_after_the_authoritative_chunk_changes() {
    let mut world = WorldState::default();
    world.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let tracker = LightingRebuildTracker {
        session_id: 4,
        ..Default::default()
    };
    let cached = CachedLightInfo {
        revision: 7,
        ..Default::default()
    };
    let result = LightingBuildResult {
        session_id: 4,
        content_revision: 7,
        world_revision: world.snapshot_revision(),
        snapshot,
        lights: HashMap::new(),
        sources: Vec::new(),
        elapsed: Duration::ZERO,
    };
    assert!(lighting_result_is_current(
        &result, &tracker, &cached, &world,
    ));

    Arc::make_mut(world.chunk_mut(IVec3::ZERO).unwrap()).set_voxel(0, 0, 0, 1);

    assert!(!lighting_result_is_current(
        &result, &tracker, &cached, &world,
    ));
}
