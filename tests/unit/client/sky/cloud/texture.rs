use super::*;

use crate::client::sky::cloud::constants::CLOUD_TEXTURE_SIZE;

#[test]
fn cloud_texture_has_expected_size_and_is_rgba() {
    let texture = generate_cloud_texture(0.55, 42);
    let (w, h) = texture.dimensions();
    assert_eq!(w, CLOUD_TEXTURE_SIZE);
    assert_eq!(h, CLOUD_TEXTURE_SIZE);
}

#[test]
fn cloud_texture_is_deterministic_for_same_seed() {
    let first = generate_cloud_texture(0.55, 20260803);
    let second = generate_cloud_texture(0.55, 20260803);
    assert_eq!(first.as_raw(), second.as_raw());
}

#[test]
fn cloud_texture_differs_across_seeds() {
    let first = generate_cloud_texture(0.55, 1);
    let second = generate_cloud_texture(0.55, 2);
    assert_ne!(first.as_raw(), second.as_raw());
}

#[test]
fn cloud_texture_is_tileable_at_edges() {
    // 可平铺性：环面坐标按 (size-1) 归一化，x=0 与 x=size-1、y=0 与 y=size-1
    // 相位一致，边缘像素应完全相同。
    let texture = generate_cloud_texture(0.55, 42);
    let size = CLOUD_TEXTURE_SIZE;
    for y in 0..size {
        assert_eq!(
            texture.get_pixel(0, y),
            texture.get_pixel(size - 1, y),
            "horizontal seam at y={y}"
        );
    }
    for x in 0..size {
        assert_eq!(
            texture.get_pixel(x, 0),
            texture.get_pixel(x, size - 1),
            "vertical seam at x={x}"
        );
    }
}

#[test]
fn higher_density_threshold_yields_fewer_cloud_pixels() {
    let sparse = generate_cloud_texture(0.75, 42);
    let dense = generate_cloud_texture(0.55, 42);
    let count_alpha = |texture: &image::RgbaImage| {
        let size = CLOUD_TEXTURE_SIZE;
        let mut count = 0u32;
        for y in 0..size {
            for x in 0..size {
                if texture.get_pixel(x, y).0[3] > 0 {
                    count += 1;
                }
            }
        }
        count
    };
    // 阈值越高云越稀疏；使用宽松比较避免边界噪声影响。
    assert!(count_alpha(&sparse) < count_alpha(&dense));
}

#[test]
fn cloud_image_uses_repeat_sampling_for_uv_drift() {
    let image = cloud_image_to_bevy(image::RgbaImage::new(2, 2));
    let ImageSampler::Descriptor(descriptor) = image.sampler else {
        panic!("cloud image must use an explicit sampler");
    };

    assert_eq!(descriptor.address_mode_u, ImageAddressMode::Repeat);
    assert_eq!(descriptor.address_mode_v, ImageAddressMode::Repeat);
}
