use super::*;
use crate::game::world::state::WorldState;

#[test]
fn plan_contains_unique_tiles_across_multiple_lod_rings() {
    let config = DistantTerrainConfig::default();
    let world = WorldState::default();
    let plan = build_distant_terrain_plan(&world, IVec3::ZERO, 8, &config);
    let unique = plan.iter().map(|spec| spec.key).collect::<HashSet<_>>();

    assert_eq!(unique.len(), plan.len());
    assert!(plan.iter().any(|spec| spec.key.lod_level == 0));
    assert!(plan.iter().any(|spec| spec.key.lod_level == 1));
    // 远景扩展到 256 区块后应包含更粗糙的外环。
    assert!(plan.iter().any(|spec| spec.key.lod_level >= 2));
    assert!(plan.iter().all(|spec| spec.sample_step_blocks > 0));
}

#[test]
fn plan_changes_stably_when_player_crosses_a_chunk_boundary() {
    let config = DistantTerrainConfig::default();
    let world = WorldState::default();
    let previous = build_distant_terrain_plan(&world, IVec3::ZERO, 8, &config);
    let next = build_distant_terrain_plan(&world, IVec3::X, 8, &config);

    let previous_keys = previous.iter().map(|spec| spec.key).collect::<HashSet<_>>();
    let next_keys = next.iter().map(|spec| spec.key).collect::<HashSet<_>>();

    assert!(!previous_keys.is_empty());
    assert!(!next_keys.is_empty());
    assert!(!previous_keys.is_disjoint(&next_keys));
    assert_ne!(previous_keys, next_keys);
}

#[test]
fn plan_keeps_tile_origins_aligned_for_negative_world_coordinates() {
    let config = DistantTerrainConfig::default();
    let world = WorldState::default();
    let plan = build_distant_terrain_plan(&world, IVec3::new(-17, 0, -33), 8, &config);

    assert!(!plan.is_empty());
    assert!(plan.iter().all(|spec| {
        spec.key.origin_chunk_x.rem_euclid(spec.key.span_chunks) == 0
            && spec.key.origin_chunk_z.rem_euclid(spec.key.span_chunks) == 0
    }));
}

#[test]
fn tile_key_stays_stable_when_player_crosses_a_chunk_boundary() {
    // 让出条件改为"该 Y 层真实区块是否加载"后，coverage_mask 完全由 WorldState 决定：
    // 空 WorldState 下跨区块不会改变 coverage_mask。核心不变量是 key 稳定（瓦片身份不随玩家
    // 移动变化），即使 coverage_mask 变化也只触发原地重建，不会销毁瓦片。
    let config = DistantTerrainConfig::default();
    let world = WorldState::default();
    let previous = build_distant_terrain_plan(&world, IVec3::ZERO, 8, &config);
    let next = build_distant_terrain_plan(&world, IVec3::X, 8, &config);

    let previous_keys: HashSet<_> = previous.iter().map(|spec| spec.key).collect();
    let next_keys: HashSet<_> = next.iter().map(|spec| spec.key).collect();
    let common_keys: Vec<_> = previous_keys.intersection(&next_keys).copied().collect();
    assert!(
        !common_keys.is_empty(),
        "跨区块后共同 key 必须非空（瓦片身份不随玩家移动销毁）"
    );
}
