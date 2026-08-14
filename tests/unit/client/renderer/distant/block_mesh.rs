use super::super::planner::{DistantTerrainTileKey, DistantTerrainTileSpec, cell_in_lod_ring};

fn test_spec() -> DistantTerrainTileSpec {
    DistantTerrainTileSpec {
        key: DistantTerrainTileKey {
            lod_level: 0,
            origin_chunk_x: 0,
            origin_chunk_z: 0,
            span_chunks: 4,
        },
        sample_step_blocks: 4,
        inner_radius_chunks: 2,
        outer_radius_chunks: 8,
        lod_inner_radius_chunks: 2,
        lod_outer_radius_chunks: 8,
        player_chunk_x: 0,
        player_chunk_z: 0,
        coverage_mask: [0; 4],
    }
}

#[test]
fn coarse_cell_is_removed_when_it_touches_a_real_near_chunk() {
    let spec = test_spec();

    assert!(!cell_in_lod_ring(spec, 0, 0));
}

#[test]
fn coarse_cell_is_kept_only_when_all_covered_chunks_are_outside_near_ring() {
    let spec = test_spec();

    assert!(cell_in_lod_ring(spec, 12, 0));
    assert!(!cell_in_lod_ring(spec, 40, 0));
}
