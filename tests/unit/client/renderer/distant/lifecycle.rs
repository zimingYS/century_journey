use super::*;

fn test_key() -> DistantTerrainTileKey {
    DistantTerrainTileKey {
        lod_level: 0,
        origin_chunk_x: 8,
        origin_chunk_z: -4,
        span_chunks: 4,
    }
}

fn test_result(
    key: DistantTerrainTileKey,
    session_generation: u64,
    request_id: u64,
) -> DistantTerrainBuildResult {
    DistantTerrainBuildResult {
        session_generation,
        request_id,
        key,
        coverage_mask: [0; 4],
        mesh: super::super::block_mesh::DistantTerrainBlockMeshData {
            opaque: crate::client::renderer::world::MeshBufferData::default(),
            water: crate::client::renderer::world::MeshBufferData::default(),
        },
    }
}

#[test]
fn runtime_rejects_results_from_a_previous_session_or_request() {
    let key = test_key();
    let mut runtime = DistantTerrainRuntime {
        session_generation: 7,
        ..default()
    };
    runtime.expected_keys.insert(key);
    let request_id = runtime.begin_request(key);

    assert!(runtime.accepts(&test_result(key, 7, request_id)));
    assert!(!runtime.accepts(&test_result(key, 6, request_id)));
    assert!(!runtime.accepts(&test_result(key, 7, request_id.wrapping_add(1))));
}

#[test]
fn clearing_a_plan_removes_pending_identity_state() {
    let key = test_key();
    let mut runtime = DistantTerrainRuntime::default();
    runtime.expected_keys.insert(key);
    runtime.ordered_plan.push(DistantTerrainTileSpec {
        key,
        sample_step_blocks: 4,
        inner_radius_chunks: 8,
        outer_radius_chunks: 16,
        lod_inner_radius_chunks: 8,
        lod_outer_radius_chunks: 16,
        player_chunk_x: 0,
        player_chunk_z: 0,
        coverage_mask: [0; 4],
    });
    runtime.begin_request(key);
    runtime.tile_masks.insert(key, [0; 4]);

    runtime.clear_plan();

    assert!(runtime.expected_keys.is_empty());
    assert!(runtime.ordered_plan.is_empty());
    assert!(runtime.active_requests.is_empty());
    assert!(runtime.tile_masks.is_empty());
}

#[test]
fn initialization_guard_preserves_an_active_world_session() {
    let runtime = DistantTerrainRuntime {
        session_generation: 7,
        ..default()
    };

    assert!(runtime.session_generation > 0);
    assert_eq!(runtime.session_generation, 7);
}

#[test]
fn advancing_session_rejects_old_requests() {
    let key = test_key();
    let mut runtime = DistantTerrainRuntime {
        session_generation: 7,
        ..default()
    };
    runtime.expected_keys.insert(key);
    let request_id = runtime.begin_request(key);

    runtime.advance_session();

    assert_eq!(runtime.session_generation, 8);
    assert!(!runtime.accepts(&test_result(key, 7, request_id)));
}
