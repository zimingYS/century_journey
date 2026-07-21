use super::*;

#[test]
fn water_top_is_lower_than_adjacent_solid_blocks() {
    let (mut positions, _) = get_merged_face_data(0, 0, 0, 1, 1, 0, 1, 0, 2, 0, 1);
    inset_water_surface(&mut positions, 0);
    assert!(
        positions
            .iter()
            .all(|position| (position[1] - (1.0 - WATER_SURFACE_INSET)).abs() < 0.0001)
    );
}

#[test]
fn water_side_keeps_its_bottom_and_lowers_only_the_top_edge() {
    let (mut positions, _) = get_merged_face_data(0, 0, 0, 1, 1, 2, 0, 2, 1, 0, 1);
    inset_water_surface(&mut positions, 2);
    let top_vertices = positions
        .iter()
        .filter(|position| position[1] > 0.0)
        .count();
    let bottom_vertices = positions
        .iter()
        .filter(|position| position[1] == 0.0)
        .count();
    assert_eq!(top_vertices, 2);
    assert_eq!(bottom_vertices, 2);
}
