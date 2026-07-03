use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::engine::asset::manager::AssetManager;
use crate::engine::constant::texture::TILE_SIZE;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::Color;
use bevy::image::{Image, ImageSampler, TextureAtlasLayout};
use bevy::log::error;
use bevy::material::AlphaMode;
use bevy::math::UVec2;
use bevy::pbr::StandardMaterial;
use bevy::prelude::default;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// 构建纹理图集
pub fn build_texture_atlas(
    registry: &mut BlockRegistry,
    unique_paths: &[String],
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
    materials: &mut Assets<StandardMaterial>,
    asset: &AssetManager,
) {
    // 获取纹理层数量，计算图集总宽度
    let layer_count = unique_paths.len() as u32;

    // 平铺式图集尺寸
    let atlas_width = CHUNK_SIZE as u32 * TILE_SIZE;
    let atlas_height = layer_count * CHUNK_SIZE as u32 * TILE_SIZE;

    // 分配像素缓冲区
    let pixel_count = atlas_width * atlas_height;
    let data_len = pixel_count as usize * 4;
    let mut atlas_data = vec![0u8; data_len];

    // 遍历所有唯一贴图路径，依次绘制到纹理图集对应图层位置
    for (layer_idx, path) in unique_paths.iter().enumerate() {
        // 通过 AssetManager 加载纹理文件
        let id = crate::engine::asset::identifier::asset_id(path);
        let image = match asset.read_file_bytes_sync(&id) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    error!("无法解码贴图 {}: {}", path, e);
                    create_missing_texture_placeholder()
                }
            },
            Err(e) => {
                error!("无法加载贴图 {}: {}", path, e);
                create_missing_texture_placeholder()
            }
        };

        // 将贴图缩放到固定方块大小
        let resized = image::imageops::resize(
            &image,
            TILE_SIZE,
            TILE_SIZE,
            image::imageops::FilterType::Nearest,
        );
        let src_pixels = resized.as_raw();

        // 层起始行
        let layer_pixel_y_start = layer_idx as u32 * CHUNK_SIZE as u32 * TILE_SIZE;

        // 将缩放后的贴图像素数据复制到图集对应图层区域
        for tile_y in 0..CHUNK_SIZE as u32 {
            for tile_x in 0..CHUNK_SIZE as u32 {
                for row in 0..TILE_SIZE {
                    let dest_x = tile_x * TILE_SIZE;
                    let dest_y = layer_pixel_y_start + tile_y * TILE_SIZE + row;

                    let src_start = (row * TILE_SIZE * 4) as usize;
                    let src_end = src_start + (TILE_SIZE * 4) as usize;
                    let dest_start = ((dest_y * atlas_width + dest_x) * 4) as usize;

                    atlas_data[dest_start..dest_start + (TILE_SIZE * 4) as usize]
                        .copy_from_slice(&src_pixels[src_start..src_end]);
                }
            }
        }
    }

    // 创建纹理图集图像
    let mut array_image = Image::new(
        Extent3d {
            width: atlas_width,
            height: atlas_height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        atlas_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );

    // 设置像素风格采样
    array_image.sampler = ImageSampler::nearest();

    // 将生成的图集添加到资源管理器，保存到方块注册表
    let texture_handle = images.add(array_image);
    registry.base_texture = texture_handle.clone();

    // 创建并保存纹理图集布局
    registry.atlas_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(TILE_SIZE),
        CHUNK_SIZE as u32,
        layer_count * CHUNK_SIZE as u32,
        None,
        None,
    ));

    // 创建不透明方块材质
    registry.opaque_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        ..default()
    });

    // 创建镂空（树叶等）方块材质
    registry.cutout_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    // 创建透明混合（玻璃等）方块材质
    registry.transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.85),
        perceptual_roughness: 0.2,
        metallic: 0.05,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
}

/// 创建紫黑格缺失贴图占位符
fn create_missing_texture_placeholder() -> image::RgbaImage {
    let mut img = image::RgbaImage::new(TILE_SIZE, TILE_SIZE);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            let color = if (x / 4 + y / 4) % 2 == 0 {
                image::Rgba([255, 0, 255, 255]) // 紫色
            } else {
                image::Rgba([0, 0, 0, 255]) // 黑色
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}
