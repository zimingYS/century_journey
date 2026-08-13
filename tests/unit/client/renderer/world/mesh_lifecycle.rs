use super::*;

#[test]
fn voxel_neighbors_allow_mesh_without_any_lighting_resource() {
    let center = IVec3::ZERO;
    let mut world = WorldState::default();
    world.insert_chunk(center, Arc::new(ChunkData::new()));
    for (direction, _) in DIRECTIONS {
        world.insert_chunk(center + direction, Arc::new(ChunkData::new()));
    }

    assert!(voxel_neighbors_ready(&world, center));
}

#[test]
fn missing_voxel_neighbor_still_defers_boundary_mesh() {
    let center = IVec3::ZERO;
    let mut world = WorldState::default();
    world.insert_chunk(center, Arc::new(ChunkData::new()));
    for (direction, _) in DIRECTIONS.into_iter().take(5) {
        world.insert_chunk(center + direction, Arc::new(ChunkData::new()));
    }

    assert!(!voxel_neighbors_ready(&world, center));
}

#[test]
fn priority_mesh_allows_pending_visible_neighbor_lighting() {
    let center = IVec3::ZERO;
    let mut world = WorldState::default();
    world.insert_chunk(center, Arc::new(ChunkData::new()));
    for (direction, _) in DIRECTIONS {
        world.insert_chunk(center + direction, Arc::new(ChunkData::new()));
    }
    let lighting = WorldLighting::default();
    let streaming = WorldStreamingConfig::default();

    assert!(voxel_neighbors_ready(&world, center));
    assert!(!visible_neighbor_lights_ready(
        &lighting, &world, &streaming, center, center,
    ));
    assert!(neighbor_lights_allow_mesh(
        true, &lighting, &world, &streaming, center, center,
    ));
    assert!(!neighbor_lights_allow_mesh(
        false, &lighting, &world, &streaming, center, center,
    ));
}

#[test]
fn absent_lighting_uses_the_mesh_fallback_instead_of_blocking() {
    let center = IVec3::ZERO;

    assert!(current_light_snapshot(None, center).is_none());
}

#[test]
fn stale_initialized_light_is_kept_as_a_temporal_mesh_fallback() {
    let center = IVec3::ZERO;
    let mut lighting = WorldLighting::default();
    let mut previous = ChunkLight::default();
    previous.mark_initialized();
    lighting.chunk_lights.insert(center, Arc::new(previous));

    assert!(current_light_snapshot(Some(&lighting), center).is_some());
}

#[test]
fn newer_mesh_request_rejects_an_older_result() {
    let position = IVec3::ZERO;
    let entity = Entity::from_bits(42);
    let mut tracker = MeshRequestTracker::default();
    let older = tracker.begin(position, entity);
    let newer = tracker.begin(position, entity);

    assert!(!tracker.is_current(position, entity, older));
    assert!(tracker.is_current(position, entity, newer));
}

#[test]
fn replacement_entity_rejects_the_previous_entities_result() {
    let position = IVec3::ZERO;
    let previous = Entity::from_bits(10);
    let replacement = Entity::from_bits(11);
    let mut tracker = MeshRequestTracker::default();
    let request = tracker.begin(position, previous);

    assert!(!tracker.is_current(position, replacement, request));
}

#[test]
fn boundary_edit_prioritizes_the_adjacent_chunk_after_the_center() {
    let affected = affected_mesh_chunks(IVec3::new(CHUNK_SIZE as i32 - 1, 3, 4));

    assert_eq!(affected, vec![IVec3::ZERO, IVec3::X]);
}
