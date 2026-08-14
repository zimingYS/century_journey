use super::super::planner::{DistantTerrainTileKey, DistantTerrainTileSpec, cell_in_lod_ring};
use crate::game::world::chunk::ChunkData;
use crate::game::world::lighting::rebuild::LightingWorldSnapshot;
use crate::game::world::state::WorldState;
use bevy::math::IVec3;
use std::sync::Arc;

fn test_spec() -> DistantTerrainTileSpec {
    DistantTerrainTileSpec {
        key: DistantTerrainTileKey {
            lod_level: 0,
            origin_chunk_x: 0,
            origin_chunk_z: 0,
            span_chunks: 4,
        },
        sample_step_blocks: 4,
        outer_radius_chunks: 8,
        lod_inner_radius_chunks: 2,
        lod_outer_radius_chunks: 8,
        player_chunk_x: 0,
        player_chunk_z: 0,
        player_chunk_y: 0,
        coverage_mask: [0; 4],
    }
}

/// 空 WorldState（无真实区块）：所有覆盖区块的 `contains_chunk` 返回 false，
/// 单元在远景环内时由远景兜底绘制，避免出现真空带。
#[test]
fn unloaded_coarse_cell_falls_back_to_distant_lod() {
    let spec = test_spec();
    let world = WorldState::default();
    let snapshot = LightingWorldSnapshot::from_world(&world);

    // 单元 (12, 0) 距玩家 (0, 0) 12×4=48 方块即 3 区块，在 outer=8 远景环内。
    assert!(cell_in_lod_ring(&snapshot, spec, 12, 0));
    // 单元 (3, 0) 距玩家 3 区块，也在远景环内。
    assert!(cell_in_lod_ring(&snapshot, spec, 3, 0));
}

/// 真实区块全部加载：单元在远景环内时让出给真实区块绘制，远景不画避免重叠。
#[test]
fn loaded_coarse_cell_yields_to_real_geometry() {
    let spec = test_spec();
    let mut world = WorldState::default();
    // 给单元 (12, 0) 覆盖的所有真实区块（X=2,3，Z=0，Y=0）插一个空区块。
    for cx in 2..=3 {
        world.insert_chunk(IVec3::new(cx, 0, 0), Arc::new(ChunkData::new()));
    }
    let snapshot = LightingWorldSnapshot::from_world(&world);

    // 单元 (12, 0) 覆盖的 XZ 区块全部已加载 → 让出。
    assert!(!cell_in_lod_ring(&snapshot, spec, 12, 0));
}

/// 远景环外（距玩家超过 outer=8 区块）的单元永远不绘制。
#[test]
fn coarse_cell_outside_lod_ring_stays_unrendered() {
    let spec = test_spec();
    let world = WorldState::default();
    let snapshot = LightingWorldSnapshot::from_world(&world);

    // 单元 (40, 0) 距玩家 40×4=160 方块即 10 区块，超过 outer=8。
    assert!(!cell_in_lod_ring(&snapshot, spec, 40, 0));
}
