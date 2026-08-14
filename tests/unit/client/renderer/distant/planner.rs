use super::*;

#[test]
fn plan_contains_unique_tiles_in_both_lod_rings() {
    let config = DistantTerrainConfig::default();
    let plan = build_distant_terrain_plan(IVec3::ZERO, 8, &config);
    let unique = plan.iter().map(|spec| spec.key).collect::<HashSet<_>>();

    assert_eq!(unique.len(), plan.len());
    assert!(plan.iter().any(|spec| spec.key.lod_level == 0));
    assert!(plan.iter().any(|spec| spec.key.lod_level == 1));
    assert!(plan.iter().all(|spec| spec.sample_step_blocks > 0));
}

#[test]
fn plan_changes_stably_when_player_crosses_a_chunk_boundary() {
    let config = DistantTerrainConfig::default();
    let previous = build_distant_terrain_plan(IVec3::ZERO, 8, &config);
    let next = build_distant_terrain_plan(IVec3::X, 8, &config);

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
    let plan = build_distant_terrain_plan(IVec3::new(-17, 0, -33), 8, &config);

    assert!(!plan.is_empty());
    assert!(plan.iter().all(|spec| {
        spec.key.origin_chunk_x.rem_euclid(spec.key.span_chunks) == 0
            && spec.key.origin_chunk_z.rem_euclid(spec.key.span_chunks) == 0
    }));
}

#[test]
fn coverage_mask_changes_while_tile_key_stays_stable() {
    let config = DistantTerrainConfig::default();
    let previous = build_distant_terrain_plan(IVec3::ZERO, 8, &config);
    let next = build_distant_terrain_plan(IVec3::X, 8, &config);

    // 跨区块移动后，近景圆盘平移会让部分瓦片的覆盖位图变化，但瓦片键只由世界
    // 坐标构成，必须保持稳定——这是避免远景瓦片被销毁重建的关键不变量。
    let mut found_changed_mask_with_stable_key = false;
    for spec in &next {
        if let Some(previous_spec) = previous.iter().find(|p| p.key == spec.key) {
            if previous_spec.coverage_mask != spec.coverage_mask {
                found_changed_mask_with_stable_key = true;
            }
        }
    }
    assert!(found_changed_mask_with_stable_key);
}
