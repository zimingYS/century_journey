use super::*;

#[test]
fn builtin_palette_grade_neutralizes_sand_and_lifts_leaves() {
    let mut sand = image::RgbaImage::from_pixel(1, 1, image::Rgba([225, 200, 143, 255]));
    grade_builtin_world_texture("textures/blocks/sand.png", &mut sand);
    assert!(sand.get_pixel(0, 0)[0] < 225);
    assert!(sand.get_pixel(0, 0)[2] > 143);

    let mut leaves = image::RgbaImage::from_pixel(1, 1, image::Rgba([69, 113, 20, 255]));
    grade_builtin_world_texture("textures/blocks/leaves.png", &mut leaves);
    assert!(leaves.get_pixel(0, 0)[1] > 113);
}

#[test]
fn custom_texture_colors_are_untouched() {
    let original = image::Rgba([10, 20, 30, 255]);
    let mut custom = image::RgbaImage::from_pixel(1, 1, original);
    grade_builtin_world_texture("textures/modded/custom.png", &mut custom);
    assert_eq!(*custom.get_pixel(0, 0), original);
}
