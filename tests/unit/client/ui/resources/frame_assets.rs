use super::*;

#[test]
fn generated_frame_is_large_enough_for_nine_slice() {
    assert!(FRAME_TEXTURE_SIZE as f32 > FRAME_SLICE * 2.0);
    let image = frame_image([1; 4], [2; 4], [3; 4]);
    assert_eq!(image.texture_descriptor.size.width, FRAME_TEXTURE_SIZE);
    assert_eq!(image.texture_descriptor.size.height, FRAME_TEXTURE_SIZE);
}
