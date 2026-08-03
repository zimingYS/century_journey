//! 生成客户端使用的程序化云纹理。
//!
//! 云纹理是单通道 alpha 图（白色云形 + 透明背景），水平与垂直方向均可平铺，
//! 供云层 quad 与 billboard 云片共用。

use crate::client::sky::cloud::constants::CLOUD_TEXTURE_SIZE;
use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use noise::{NoiseFn, Perlin};

/// 生成可平铺的云形 alpha 纹理。
///
/// 使用 sin/cos 周期坐标将 2D 噪声映射到环面，保证纹理四边无缝；
/// FBM 从频率 1 起步逐倍频叠加，配合较高的密度阈值让云形更稀疏自然，
/// 边缘用 smoothstep 软过渡避免硬边。
pub fn generate_cloud_texture(density: f32, seed: u32) -> image::RgbaImage {
    let size = CLOUD_TEXTURE_SIZE;
    let mut img = image::RgbaImage::new(size, size);
    let perlin = Perlin::new(seed);

    // 内容定义直接控制覆盖阈值；极值可用于完全阴天或近乎无云的配置。
    let threshold = density.clamp(0.0, 1.0);
    // 边缘过渡区间 0.18：阈值上下各 0.09 用于 smoothstep，软化云形轮廓。
    let edge_half = 0.09_f64;

    let span = (size.saturating_sub(1)) as f32;
    for y in 0..size {
        for x in 0..size {
            // 环面坐标：按 (size-1) 归一化使 x=0 与 x=size-1 相位一致。
            let u = (x as f32 / span) * std::f32::consts::TAU;
            let v = (y as f32 / span) * std::f32::consts::TAU;

            // FBM：5 倍频，从频率 1 起步，覆盖范围更大、细节更丰富。
            let mut sum = 0.0_f64;
            let mut amp = 1.0_f64;
            let mut freq = 1.0_f64;
            let mut norm = 0.0_f64;
            for _ in 0..5 {
                let u = u as f64;
                let v = v as f64;
                sum += perlin.get([
                    u.cos() * freq,
                    u.sin() * freq,
                    v.cos() * freq,
                    v.sin() * freq,
                ]) * amp;
                norm += amp;
                amp *= 0.55;
                freq *= 2.0;
            }
            let noise = sum / norm * 0.5 + 0.5;

            // smoothstep 软边映射：阈值附近产生柔和过渡。
            let distance = (noise - threshold as f64 + edge_half) / (2.0 * edge_half);
            let cloud = smoothstep(0.0, 1.0, distance);
            let alpha = (cloud * 255.0).round() as u8;
            img.put_pixel(x, y, image::Rgba([255, 255, 255, alpha]));
        }
    }
    img
}

/// 三次平滑过渡。
fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// 将云纹理转换为可上传到渲染器的 Bevy 图像，并使用线性采样。
pub fn cloud_image_to_bevy(img: image::RgbaImage) -> Image {
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
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        address_mode_w: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        mipmap_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/sky/cloud/texture.rs"]
mod tests;
