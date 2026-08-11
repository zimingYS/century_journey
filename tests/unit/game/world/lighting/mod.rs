use super::*;
use std::sync::Arc;

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
fn streaming_changes_are_coalesced_until_the_world_is_stable() {
    let signature = vec![(IVec3::ZERO, 1)];
    let mut tracker = LightingRebuildTracker::default();

    tracker.observe(signature.clone(), 0);
    assert!(!tracker.ready_to_dispatch(0));
    for _ in 1..WORLD_REBUILD_STABLE_TICKS {
        tracker.observe(signature.clone(), 0);
        assert!(!tracker.ready_to_dispatch(0));
    }
    tracker.observe(signature, 0);

    assert!(tracker.ready_to_dispatch(0));
    assert!(!tracker.ready_to_dispatch(1));
}

#[test]
fn content_changes_bypass_the_streaming_stability_delay() {
    let mut tracker = LightingRebuildTracker::default();

    tracker.observe(Vec::new(), 1);

    assert!(tracker.ready_to_dispatch(0));
}

#[test]
fn result_is_rejected_after_the_authoritative_chunk_changes() {
    let mut world = WorldState::default();
    world.insert_chunk(IVec3::ZERO, Arc::new(ChunkData::new()));
    let snapshot = LightingWorldSnapshot::from_world(&world);
    let tracker = LightingRebuildTracker {
        session_id: 4,
        ..default()
    };
    let cached = CachedLightInfo {
        revision: 7,
        ..default()
    };
    let result = LightingBuildResult {
        session_id: 4,
        content_revision: 7,
        snapshot,
        lights: HashMap::new(),
        sources: Vec::new(),
        elapsed: Duration::ZERO,
    };
    assert!(lighting_result_is_current(
        &result,
        &tracker,
        &cached,
        &current_world_signature(&world),
    ));

    Arc::make_mut(world.chunk_mut(IVec3::ZERO).unwrap()).set_voxel(0, 0, 0, 1);

    assert!(!lighting_result_is_current(
        &result,
        &tracker,
        &cached,
        &current_world_signature(&world),
    ));
}
