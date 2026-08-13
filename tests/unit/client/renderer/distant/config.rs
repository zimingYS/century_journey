use super::*;

#[test]
fn far_radius_scales_with_near_mesh_radius_and_stays_bounded() {
    let config = DistantTerrainConfig::default();

    assert_eq!(config.far_radius_chunks(2), 16);
    assert_eq!(config.far_radius_chunks(8), 32);
    assert_eq!(config.far_radius_chunks(24), 64);
    assert_eq!(config.far_radius_chunks(200), 64);
}

#[test]
fn view_distance_keeps_room_for_the_outer_tile_span() {
    let config = DistantTerrainConfig::default();
    let radius_only = config.far_radius_chunks(8) as f32 * CHUNK_SIZE as f32;

    assert!(config.view_distance_blocks(8) > radius_only);
    assert_eq!(config.fog_distance_blocks(8), radius_only);
}
