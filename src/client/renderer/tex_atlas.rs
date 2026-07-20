use crate::content::block::definition::RenderMode;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::engine::constant::texture::TILE_SIZE;
use bevy::asset::{Assets, RenderAssetUsages};
use bevy::color::Color;
use bevy::image::{Image, ImageSampler, TextureAtlasLayout};
use bevy::log::error;
use bevy::material::AlphaMode;
use bevy::math::UVec2;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[derive(Resource, Clone)]
pub struct BlockRenderAssets {
    base_texture: Handle<Image>,
    atlas_layout: Handle<TextureAtlasLayout>,
    opaque_material: Handle<StandardMaterial>,
    cutout_material: Handle<StandardMaterial>,
    transparent_material: Handle<StandardMaterial>,
}

impl BlockRenderAssets {
    pub fn base_texture(&self) -> &Handle<Image> {
        &self.base_texture
    }

    pub fn atlas_layout(&self) -> &Handle<TextureAtlasLayout> {
        &self.atlas_layout
    }

    pub fn material(&self, mode: RenderMode) -> &Handle<StandardMaterial> {
        match mode {
            RenderMode::Opaque => &self.opaque_material,
            RenderMode::Transparent => &self.transparent_material,
            _ => &self.cutout_material,
        }
    }

    pub fn opaque_material(&self) -> &Handle<StandardMaterial> {
        &self.opaque_material
    }

    pub fn cutout_material(&self) -> &Handle<StandardMaterial> {
        &self.cutout_material
    }

    pub fn transparent_material(&self) -> &Handle<StandardMaterial> {
        &self.transparent_material
    }
}

pub fn init_block_render_assets_system(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset: Res<AssetManager>,
) {
    let render_assets =
        build_texture_atlas(&registry, &mut images, &mut layouts, &mut materials, &asset);
    commands.insert_resource(render_assets);
}

pub fn build_texture_atlas(
    registry: &BlockRegistry,
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
    materials: &mut Assets<StandardMaterial>,
    asset: &AssetManager,
) -> BlockRenderAssets {
    let unique_paths = registry.texture_paths();
    let layer_count = unique_paths.len() as u32;

    let atlas_width = CHUNK_SIZE as u32 * TILE_SIZE;
    let atlas_height = layer_count * CHUNK_SIZE as u32 * TILE_SIZE;

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
            TILE_SIZE,
            TILE_SIZE,
            image::imageops::FilterType::Nearest,
        );
        let src_pixels = resized.as_raw();

        let layer_pixel_y_start = layer_idx as u32 * CHUNK_SIZE as u32 * TILE_SIZE;

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
        UVec2::splat(TILE_SIZE),
        CHUNK_SIZE as u32,
        layer_count * CHUNK_SIZE as u32,
        None,
        None,
    ));

    let opaque_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        ..default()
    });

    let cutout_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        perceptual_roughness: 0.85,
        alpha_mode: AlphaMode::Mask(0.5),
        ..default()
    });

    let transparent_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        base_color: Color::srgba(0.76, 0.90, 1.0, 0.72),
        perceptual_roughness: 0.12,
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });

    BlockRenderAssets {
        base_texture: texture_handle,
        atlas_layout,
        opaque_material,
        cutout_material,
        transparent_material,
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
    let mut img = image::RgbaImage::new(TILE_SIZE, TILE_SIZE);
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
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
mod tests {
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
}
