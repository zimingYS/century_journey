//! 从内容纹理构建方块图集，并提供运行时纹理索引映射。

use crate::client::renderer::constants::{BLOCK_ATLAS_TILES_PER_ROW, BLOCK_TILE_SIZE};
use crate::client::renderer::lighting::material::{VoxelMaterial, VoxelMaterialExtension};
use crate::client::water::{WaterMaterial, WaterMaterialExtension};
use crate::content::block::definition::RenderMode;
use crate::content::block::registry::BlockRegistry;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::Color;
use bevy::ecs::system::SystemParam;
use bevy::image::{Image, ImageSampler, TextureAtlasLayout};
use bevy::log::error;
use bevy::material::AlphaMode;
use bevy::math::UVec2;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// 方块图集及各透明模式共享材质的客户端资产集合。
#[derive(Resource, Clone)]
pub struct BlockRenderAssets {
    base_texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    opaque_material: Handle<StandardMaterial>,
    cutout_material: Handle<StandardMaterial>,
    transparent_material: Handle<StandardMaterial>,
    voxel_opaque_material: Handle<VoxelMaterial>,
    voxel_cutout_material: Handle<VoxelMaterial>,
    /// 水面可见性基线材质：不依赖扩展着色器，保证水体始终可见。
    water_base_material: Handle<StandardMaterial>,
    /// 叠加在基线之上的动态水面效果材质。
    water_effect_material: Handle<WaterMaterial>,
}

/// 聚合方块图集初始化所需的三类材质资产池。
#[derive(SystemParam)]
pub(crate) struct BlockMaterialAssetParams<'w> {
    standard: ResMut<'w, Assets<StandardMaterial>>,
    voxel: ResMut<'w, Assets<VoxelMaterial>>,
    water: ResMut<'w, Assets<WaterMaterial>>,
}

impl BlockRenderAssets {
    /// 返回方块图集基础纹理。
    pub fn base_texture(&self) -> &Handle<Image> {
        &self.base_texture
    }

    /// 返回方块图集布局。
    pub fn atlas_layout(&self) -> &Handle<TextureAtlasLayout> {
        &self.atlas_layout
    }

    /// 返回指定渲染模式使用的共享材质。
    pub fn material(&self, mode: RenderMode) -> &Handle<StandardMaterial> {
        match mode {
            RenderMode::Opaque => &self.opaque_material,
            RenderMode::Transparent => &self.transparent_material,
            _ => &self.cutout_material,
        }
    }

    /// 返回不透明方块材质。
    pub fn opaque_material(&self) -> &Handle<StandardMaterial> {
        &self.opaque_material
    }

    /// 返回透明裁切方块材质。
    pub fn cutout_material(&self) -> &Handle<StandardMaterial> {
        &self.cutout_material
    }

    /// 返回半透明方块材质。
    pub fn transparent_material(&self) -> &Handle<StandardMaterial> {
        &self.transparent_material
    }

    /// 返回携带独立方块光的世界不透明区块材质。
    pub(crate) fn voxel_opaque_material(&self) -> &Handle<VoxelMaterial> {
        &self.voxel_opaque_material
    }

    /// 返回携带独立方块光的世界透明裁切区块材质。
    pub(crate) fn voxel_cutout_material(&self) -> &Handle<VoxelMaterial> {
        &self.voxel_cutout_material
    }

    /// 返回不依赖扩展着色器的水面可见性基线材质。
    pub fn water_base_material(&self) -> &Handle<StandardMaterial> {
        &self.water_base_material
    }

    /// 返回负责深度、高光和泡沫的动态水面效果材质。
    pub fn water_effect_material(&self) -> &Handle<WaterMaterial> {
        &self.water_effect_material
    }
}

/// 在内容注册表就绪后构建并插入方块渲染资产资源。
pub(crate) fn init_block_render_assets_system(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    materials: BlockMaterialAssetParams,
    asset: Res<AssetManager>,
) {
    let BlockMaterialAssetParams {
        mut standard,
        mut voxel,
        mut water,
    } = materials;
    let render_assets = build_texture_atlas(
        &registry,
        &mut images,
        &mut layouts,
        &mut standard,
        &mut voxel,
        &mut water,
        &asset,
    );
    commands.insert_resource(render_assets);
}

/// 从方块注册表引用的纹理构建图集、布局及三类共享材质。
pub(crate) fn build_texture_atlas(
    registry: &BlockRegistry,
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
    materials: &mut Assets<StandardMaterial>,
    voxel_materials: &mut Assets<VoxelMaterial>,
    water_materials: &mut Assets<WaterMaterial>,
    asset: &AssetManager,
) -> BlockRenderAssets {
    let unique_paths = registry.texture_paths();
    let layer_count = unique_paths.len() as u32;

    let atlas_width = BLOCK_ATLAS_TILES_PER_ROW * BLOCK_TILE_SIZE;
    let atlas_height = layer_count * BLOCK_ATLAS_TILES_PER_ROW * BLOCK_TILE_SIZE;

    let pixel_count = atlas_width * atlas_height;
    let data_len = pixel_count as usize * 4;
    let mut atlas_data = vec![0u8; data_len];

    let files = AssetFiles::new(asset.resolver());

    for (layer_idx, path) in unique_paths.iter().enumerate() {
        let id = crate::engine::asset::identifier::asset_id(path);
        let mut image = match files.read_bytes(&id) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(e) => {
                    error!("cannot decode block texture {path}: {e}");
                    create_missing_texture_placeholder()
                }
            },
            Err(e) => {
                error!("cannot load block texture {path}: {e}");
                create_missing_texture_placeholder()
            }
        };
        grade_builtin_world_texture(path, &mut image);

        let resized = image::imageops::resize(
            &image,
            BLOCK_TILE_SIZE,
            BLOCK_TILE_SIZE,
            image::imageops::FilterType::Nearest,
        );
        let src_pixels = resized.as_raw();

        let layer_pixel_y_start = layer_idx as u32 * BLOCK_ATLAS_TILES_PER_ROW * BLOCK_TILE_SIZE;

        for tile_y in 0..BLOCK_ATLAS_TILES_PER_ROW {
            for tile_x in 0..BLOCK_ATLAS_TILES_PER_ROW {
                for row in 0..BLOCK_TILE_SIZE {
                    let dest_x = tile_x * BLOCK_TILE_SIZE;
                    let dest_y = layer_pixel_y_start + tile_y * BLOCK_TILE_SIZE + row;

                    let src_start = (row * BLOCK_TILE_SIZE * 4) as usize;
                    let src_end = src_start + (BLOCK_TILE_SIZE * 4) as usize;
                    let dest_start = ((dest_y * atlas_width + dest_x) * 4) as usize;

                    atlas_data[dest_start..dest_start + (BLOCK_TILE_SIZE * 4) as usize]
                        .copy_from_slice(&src_pixels[src_start..src_end]);
                }
            }
        }
    }

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
    array_image.sampler = ImageSampler::nearest();

    let texture_handle = images.add(array_image);
    let atlas_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::splat(BLOCK_TILE_SIZE),
        BLOCK_ATLAS_TILES_PER_ROW,
        layer_count * BLOCK_ATLAS_TILES_PER_ROW,
        None,
        None,
    ));

    let opaque_base = StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        ..default()
    };
    let voxel_opaque_material = voxel_materials.add(VoxelMaterial {
        base: opaque_base.clone(),
        extension: VoxelMaterialExtension::default(),
    });
    let opaque_material = materials.add(opaque_base);

    let cutout_base = StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    };
    let voxel_cutout_material = voxel_materials.add(VoxelMaterial {
        base: cutout_base.clone(),
        extension: VoxelMaterialExtension::default(),
    });
    let cutout_material = materials.add(cutout_base);

    let transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        base_color: Color::srgba(0.76, 0.90, 1.0, 0.72),
        perceptual_roughness: 0.12,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    // 水面独立材质：单独加载 water.png 为 repeat 纹理，避免 uv_transform
    // 平移时采样到图集相邻 tile。
    let (water_base_material, water_effect_material) = {
        let water_path = "textures/blocks/water.png";
        let water_id = crate::engine::asset::identifier::asset_id(water_path);
        let mut water_image = match files.read_bytes(&water_id) {
            Ok(bytes) => match image::load_from_memory(&bytes) {
                Ok(img) => img.to_rgba8(),
                Err(_) => {
                    error!("cannot decode water texture {water_path}");
                    image::RgbaImage::new(BLOCK_TILE_SIZE, BLOCK_TILE_SIZE)
                }
            },
            Err(_) => {
                error!("cannot load water texture {water_path}");
                image::RgbaImage::new(BLOCK_TILE_SIZE, BLOCK_TILE_SIZE)
            }
        };
        // 水位调整：轻度提升亮度和饱和度，让水面在暗环境可读。
        for pixel in water_image.pixels_mut() {
            for channel in 0..3 {
                pixel[channel] = (pixel[channel] as f32 * 1.1 + 6.0).min(255.0) as u8;
            }
        }
        let resized = image::imageops::resize(
            &water_image,
            BLOCK_TILE_SIZE,
            BLOCK_TILE_SIZE,
            image::imageops::FilterType::Nearest,
        );
        let data = resized.into_raw();
        let mut water_texture = Image::new(
            Extent3d {
                width: BLOCK_TILE_SIZE,
                height: BLOCK_TILE_SIZE,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        // 平铺采样供水面 shader 叠加时空波形；线性过滤保持高光连续。
        let mut sampler = bevy::image::ImageSamplerDescriptor::linear();
        sampler.address_mode_u = bevy::render::render_resource::AddressMode::Repeat.into();
        sampler.address_mode_v = bevy::render::render_resource::AddressMode::Repeat.into();
        water_texture.sampler = bevy::image::ImageSampler::Descriptor(sampler);
        let water_handle = images.add(water_texture);
        let base_material = materials.add(StandardMaterial {
            base_color_texture: Some(water_handle.clone()),
            base_color: Color::srgba(0.58, 0.86, 1.0, 0.74),
            perceptual_roughness: 0.16,
            reflectance: 0.86,
            metallic: 0.0,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        let effect_material = water_materials.add(WaterMaterial {
            base: StandardMaterial {
                base_color_texture: Some(water_handle),
                base_color: Color::srgba(0.98, 1.0, 1.0, 0.32),
                perceptual_roughness: 0.10,
                reflectance: 0.98,
                metallic: 0.0,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            },
            extension: WaterMaterialExtension::default(),
        });
        (base_material, effect_material)
    };

    BlockRenderAssets {
        base_texture: texture_handle,
        atlas_layout,
        opaque_material,
        cutout_material,
        transparent_material,
        voxel_opaque_material,
        voxel_cutout_material,
        water_base_material,
        water_effect_material,
    }
}

fn grade_builtin_world_texture(path: &str, image: &mut image::RgbaImage) {
    let normalized = path.replace('\\', "/");
    let (gain, lift) = match normalized.as_str() {
        "textures/blocks/sand.png" => ([0.90, 0.94, 1.10], [4.0, 4.0, 4.0]),
        "textures/blocks/leaves.png" => ([1.15, 1.22, 1.35], [6.0, 6.0, 6.0]),
        _ => return,
    };

    for pixel in image.pixels_mut() {
        if pixel[3] == 0 {
            continue;
        }
        for channel in 0..3 {
            pixel[channel] = (pixel[channel] as f32 * gain[channel] + lift[channel])
                .round()
                .clamp(0.0, 255.0) as u8;
        }
    }
}

fn create_missing_texture_placeholder() -> image::RgbaImage {
    let mut img = image::RgbaImage::new(BLOCK_TILE_SIZE, BLOCK_TILE_SIZE);
    for y in 0..BLOCK_TILE_SIZE {
        for x in 0..BLOCK_TILE_SIZE {
            let color = if (x / 4 + y / 4) % 2 == 0 {
                image::Rgba([255, 0, 255, 255])
            } else {
                image::Rgba([0, 0, 0, 255])
            };
            img.put_pixel(x, y, color);
        }
    }
    img
}

#[cfg(test)]
#[path = "../../../tests/unit/client/renderer/tex_atlas.rs"]
mod tests;
