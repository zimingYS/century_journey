use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// 生成太阳纹理（径向渐变发光圆盘）
pub fn generate_sun_texture(size: u32) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(size, size);
    let center = (size.saturating_sub(1)) as f32 * 0.5;
    let radius = size as f32 * 0.5;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let normalized = (dx * dx + dy * dy).sqrt() / radius;

            let pixel = if normalized <= 0.50 {
                image::Rgba([255, 250, 222, 255])
            } else if normalized <= 0.61 {
                let t = smoothstep((normalized - 0.50) / 0.11);
                image::Rgba([
                    255,
                    lerp_u8(250, 224, t),
                    lerp_u8(222, 155, t),
                    lerp_u8(255, 220, t),
                ])
            } else if normalized <= 0.94 {
                let t = smoothstep((normalized - 0.61) / 0.33);
                image::Rgba([
                    255,
                    lerp_u8(224, 196, t),
                    lerp_u8(155, 116, t),
                    lerp_u8(82, 0, t),
                ])
            } else {
                image::Rgba([0, 0, 0, 0])
            };
            img.put_pixel(x, y, pixel);
        }
    }

    img
}

fn lerp_u8(from: u8, to: u8, amount: f32) -> u8 {
    (from as f32 + (to as f32 - from as f32) * amount.clamp(0.0, 1.0)).round() as u8
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

/// 生成月亮纹理
pub fn generate_moon_texture(size: u32) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(size, size);
    let center = size as f32 / 2.0;

    // 方形核心半径
    let core_half = size as f32 * 0.40;

    // 预设环形山位置
    let craters: [(f32, f32, f32, f32); 9] = [
        (-0.28, -0.30, 0.08, 0.14),
        (0.10, -0.32, 0.10, 0.16),
        (0.30, -0.18, 0.07, 0.12),
        (-0.32, 0.08, 0.09, 0.13),
        (-0.08, 0.02, 0.16, 0.10),
        (0.32, 0.12, 0.06, 0.10),
        (-0.22, 0.30, 0.07, 0.11),
        (0.12, 0.28, 0.08, 0.12),
        (0.26, 0.32, 0.05, 0.09),
    ];

    for y in 0..size {
        for x in 0..size {
            let nx = (x as f32 - center) / center;
            let ny = (y as f32 - center) / center;
            let chebyshev = nx.abs().max(ny.abs());

            if chebyshev <= core_half / center {
                // 基础灰白色
                let mut brightness = 215.0f32;

                // 环形山暗化
                for (cx, cy, cr, dark) in &craters {
                    let cdist = ((nx - cx).powi(2) + (ny - cy).powi(2)).sqrt();
                    if cdist < *cr {
                        let falloff = 1.0 - cdist / cr;
                        brightness -= dark * falloff * 255.0;
                    }
                }

                // 边缘微暗
                let edge_factor = 1.0 - chebyshev * center / core_half * 0.12;
                brightness *= edge_factor;

                let v = brightness.clamp(70.0, 255.0) as u8;
                img.put_pixel(
                    x,
                    y,
                    image::Rgba([v, v, (v as f32 * 1.04).min(255.0) as u8, 255]),
                );
            } else {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            }
        }
    }

    img
}

/// 生成星星纹理（发光点）
pub fn generate_star_texture(size: u32) -> image::RgbaImage {
    let mut img = image::RgbaImage::new(size, size);
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();

            let alpha = if dist <= radius * 0.3 {
                255u8
            } else if dist <= radius {
                let t = (dist - radius * 0.3) / (radius * 0.7);
                ((1.0 - t) * 255.0) as u8
            } else {
                0
            };

            img.put_pixel(x, y, image::Rgba([255, 255, 245, alpha]));
        }
    }

    img
}

pub fn rgba_image_to_bevy(img: image::RgbaImage) -> Image {
    let (w, h) = img.dimensions();
    let data = img.into_raw();
    let mut image = Image::new(
        Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::linear();
    image
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_has_a_round_opaque_core_and_transparent_corners() {
        let texture = generate_sun_texture(128);
        assert_eq!(texture.get_pixel(64, 64).0[3], 255);
        assert_eq!(texture.get_pixel(0, 0).0[3], 0);
        assert_eq!(texture.get_pixel(64, 0).0[3], 0);
    }
}
