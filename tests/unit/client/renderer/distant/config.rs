use super::*;

#[test]
fn far_radius_scales_with_near_mesh_radius_and_stays_bounded() {
    let config = DistantTerrainConfig::default();

    assert_eq!(config.far_radius_chunks(2), 64);
    assert_eq!(config.far_radius_chunks(8), 256);
    assert_eq!(config.far_radius_chunks(24), 256);
    assert_eq!(config.far_radius_chunks(200), 256);
}

#[test]
fn view_distance_keeps_room_for_the_outer_tile_span() {
    let config = DistantTerrainConfig::default();
    let radius_only = config.far_radius_chunks(8) as f32 * CHUNK_SIZE as f32;

    assert!(config.view_distance_blocks(8) > radius_only);
    assert_eq!(config.fog_distance_blocks(8), radius_only);
}

#[test]
fn rings_generate_more_levels_for_farther_view_distance() {
    let config = DistantTerrainConfig::default();
    let rings = config.rings(8);

    // 默认近景半径下应生成至少四级环，覆盖到 256 区块的最远视野。
    assert!(rings.len() >= 4);
    assert_eq!(rings.first().unwrap().inner_radius_chunks, 8);
    assert_eq!(rings.last().unwrap().outer_radius_chunks, 256);
    // 相邻环边界衔接，且越往外瓦片跨度越大（越粗糙）。
    for pair in rings.windows(2) {
        assert_eq!(pair[0].outer_radius_chunks, pair[1].inner_radius_chunks);
        assert!(pair[1].tile_span_chunks > pair[0].tile_span_chunks);
    }
}
