use bevy::asset::RenderAssetUsages;
use bevy::camera::{RenderTarget, ScalingMode, visibility::RenderLayers};
use bevy::image::ImageSampler;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::client::renderer::item::baked_model::BakedItemModel;
use crate::client::renderer::item::display::ItemDisplayContext;
use crate::client::renderer::item::gui_icon_cache::{GuiItemIcon, GuiItemIconCache};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::content::block::registry::BlockRegistry;
use crate::content::constant::world::CHUNK_SIZE;
use crate::engine::constant::texture::TILE_SIZE;
use crate::shared::identifier::Identifier;

/// GUI 图标专用渲染层，避免离屏相机把主世界也渲进去。
const ITEM_MODEL_PREVIEW_LAYER: usize = 6;
/// GUI 方块图标生成尺寸。
const ITEM_MODEL_PREVIEW_SIZE: u32 = 96;
/// 多个离屏图标临时场景之间的空间间隔。
const ITEM_MODEL_PREVIEW_SPACING: f32 = 6.0;
/// GUI 图标相机距离。
const ITEM_MODEL_CAMERA_DISTANCE: f32 = 4.0;
/// GUI 图标正交相机视野大小。
const ITEM_MODEL_ORTHO_SIZE: f32 = 1.62;
/// 离屏相机保留的帧数。
///
/// 这个路径现在只作为未来 custom / glTF 模型的兜底；方块图标已经改为 CPU 烘焙，避免大量临时相机造成帧率下降。
const ICON_CAMERA_LIFETIME_FRAMES: u8 = 90;
/// 方块图标左侧面的固定亮度。
const BLOCK_ICON_LEFT_BRIGHTNESS: f32 = 0.72;
/// 方块图标右侧面的固定亮度。
const BLOCK_ICON_RIGHT_BRIGHTNESS: f32 = 0.86;
/// 方块图标顶面的固定亮度。
const BLOCK_ICON_TOP_BRIGHTNESS: f32 = 1.0;
/// 方块图标的水平半宽。
const BLOCK_ICON_HALF_WIDTH: f32 = 30.0;
/// 顶面菱形的垂直半高。
const BLOCK_ICON_TOP_HALF_HEIGHT: f32 = 15.0;
/// 侧面的垂直高度。
const BLOCK_ICON_SIDE_HEIGHT: f32 = 40.0;
/// 方块图标的视觉中心，略低于纹理中心以贴近 Minecraft 槽位观感。
const BLOCK_ICON_VISUAL_CENTER_Y: f32 = 52.0;

/// 标记一个临时 GUI 图标相机。
#[derive(Component)]
pub struct GuiItemIconBakeCamera {
    /// 当前图标对应的物品 ID。
    item_identifier: Identifier,
    /// 还需要继续渲染的帧数。
    frames_left: u8,
    /// 图标模型根实体，烘焙完成后会连同子节点一起删除。
    root: Entity,
}

/// 把已经烘焙好的 BakedItemModel 渲染成 GUI Image。
pub struct GuiItemIconBaker;

impl GuiItemIconBaker {
    /// 为一个方块物品生成 Minecraft 风格的等轴测 GUI 图标。
    ///
    /// 这里直接从方块 atlas 的顶面、正面、右面贴图采样并在 CPU 上合成透明 Image，避免为每个方块创建离屏 3D 相机。
    pub fn bake_block_cube_icon(
        item_identifier: &Identifier,
        block_identifier: &Identifier,
        block_registry: &BlockRegistry,
        render_assets: &BlockRenderAssets,
        cache: &mut GuiItemIconCache,
        images: &mut Assets<Image>,
    ) -> Option<Handle<Image>> {
        if let Some(image) = cache.cached_icon_image(item_identifier) {
            return Some(image);
        }

        let runtime_id = block_registry.get_id_by_identifier(&block_identifier.to_string())?;
        let face_layers = [
            block_registry.get_layer(runtime_id, 0),
            block_registry.get_layer(runtime_id, 4),
            block_registry.get_layer(runtime_id, 3),
        ];

        let icon_pixels = {
            let atlas_image = images.get(render_assets.base_texture())?;
            let atlas_data = atlas_image.data.as_ref()?;
            let atlas_size = atlas_image.texture_descriptor.size;
            render_block_cube_icon(atlas_data, atlas_size, face_layers)
        };

        let mut image = Image::new(
            Extent3d {
                width: ITEM_MODEL_PREVIEW_SIZE,
                height: ITEM_MODEL_PREVIEW_SIZE,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            icon_pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );
        image.sampler = ImageSampler::nearest();
        let image_handle = images.add(image);

        cache.insert_icon(
            item_identifier.clone(),
            GuiItemIcon {
                image: image_handle.clone(),
                ready: true,
            },
        );

        Some(image_handle)
    }

    /// 为一个通用 3D 模型创建离屏 GUI 图标。
    ///
    /// 方块不再走这个路径；它保留给以后真正的 custom model、glTF 或复杂 JSON model 使用。
    pub fn bake(
        item_identifier: &Identifier,
        model: &BakedItemModel,
        cache: &mut GuiItemIconCache,
        commands: &mut Commands,
        images: &mut Assets<Image>,
        materials: &mut Assets<StandardMaterial>,
    ) -> Option<Handle<Image>> {
        if let Some(image) = cache.cached_icon_image(item_identifier) {
            return Some(image);
        }
        if model.is_empty() {
            return None;
        }

        let image = Image::new_target_texture(
            ITEM_MODEL_PREVIEW_SIZE,
            ITEM_MODEL_PREVIEW_SIZE,
            TextureFormat::Rgba8UnormSrgb,
            None,
        );
        let image_handle = images.add(image);

        let preview_index = cache.next_icon_index() as f32;
        let target = Vec3::new(preview_index * ITEM_MODEL_PREVIEW_SPACING, -500.0, 0.0);
        let camera_pos = target + Vec3::new(0.0, 0.0, ITEM_MODEL_CAMERA_DISTANCE);
        let preview_layer = RenderLayers::layer(ITEM_MODEL_PREVIEW_LAYER);

        let root = commands
            .spawn((
                Name::new(format!("GuiItemIcon_{item_identifier}")),
                Transform::from_translation(target),
                Visibility::Inherited,
                preview_layer.clone(),
            ))
            .id();

        let model_root = commands
            .spawn((
                Name::new("GuiItemIconModel"),
                model.display_transform(ItemDisplayContext::Gui),
                Visibility::Inherited,
                preview_layer.clone(),
            ))
            .id();
        commands.entity(root).add_child(model_root);

        for part in &model.parts {
            let material = preview_material(&part.material, materials);
            let part_entity = commands
                .spawn((
                    Name::new(part.name.clone()),
                    Mesh3d(part.mesh.clone()),
                    MeshMaterial3d(material),
                    part.transform,
                    Visibility::Inherited,
                    NotShadowCaster,
                    preview_layer.clone(),
                ))
                .id();
            commands.entity(model_root).add_child(part_entity);
        }

        commands.spawn((
            Name::new(format!("GuiItemIconCamera_{item_identifier}")),
            Camera3d::default(),
            Camera {
                order: -10,
                clear_color: Color::NONE.into(),
                ..default()
            },
            RenderTarget::Image(image_handle.clone().into()),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::Fixed {
                    width: ITEM_MODEL_ORTHO_SIZE,
                    height: ITEM_MODEL_ORTHO_SIZE,
                },
                near: 0.0,
                far: 32.0,
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(camera_pos).looking_at(target, Vec3::Y),
            preview_layer,
            GuiItemIconBakeCamera {
                item_identifier: item_identifier.clone(),
                frames_left: ICON_CAMERA_LIFETIME_FRAMES,
                root,
            },
        ));

        cache.insert_icon(
            item_identifier.clone(),
            GuiItemIcon {
                image: image_handle.clone(),
                ready: false,
            },
        );

        Some(image_handle)
    }
}

/// 删除已经完成渲染的 GUI 图标临时相机和临时模型。
///
/// 图标渲染目标会保留下来给 UI 使用；临时实体移除后不再参与每帧渲染和光照计算。
pub fn retire_gui_item_icon_cameras_system(
    mut commands: Commands,
    mut cache: ResMut<GuiItemIconCache>,
    mut query: Query<(Entity, &mut GuiItemIconBakeCamera)>,
) {
    for (camera_entity, mut bake_camera) in &mut query {
        if bake_camera.frames_left > 0 {
            bake_camera.frames_left -= 1;
            continue;
        }

        cache.mark_icon_ready(&bake_camera.item_identifier);
        commands
            .entity(bake_camera.root)
            .despawn_related::<Children>()
            .despawn();
        commands.entity(camera_entity).despawn();
    }
}

/// 在 CPU 上合成一个等轴测方块图标。
fn render_block_cube_icon(
    atlas_data: &[u8],
    atlas_size: Extent3d,
    face_layers: [u32; 3],
) -> Vec<u8> {
    let mut output = vec![0; (ITEM_MODEL_PREVIEW_SIZE * ITEM_MODEL_PREVIEW_SIZE * 4) as usize];
    let center_x = ITEM_MODEL_PREVIEW_SIZE as f32 * 0.5;
    let top_y = BLOCK_ICON_VISUAL_CENTER_Y
        - (BLOCK_ICON_TOP_HALF_HEIGHT * 2.0 + BLOCK_ICON_SIDE_HEIGHT) * 0.5;

    let top = Vec2::new(center_x, top_y);
    let right = Vec2::new(
        center_x + BLOCK_ICON_HALF_WIDTH,
        top_y + BLOCK_ICON_TOP_HALF_HEIGHT,
    );
    let middle = Vec2::new(center_x, top_y + BLOCK_ICON_TOP_HALF_HEIGHT * 2.0);
    let left = Vec2::new(
        center_x - BLOCK_ICON_HALF_WIDTH,
        top_y + BLOCK_ICON_TOP_HALF_HEIGHT,
    );
    let right_down = right + Vec2::Y * BLOCK_ICON_SIDE_HEIGHT;
    let middle_down = middle + Vec2::Y * BLOCK_ICON_SIDE_HEIGHT;
    let left_down = left + Vec2::Y * BLOCK_ICON_SIDE_HEIGHT;

    let sampler = AtlasSampler {
        data: atlas_data,
        width: atlas_size.width,
        height: atlas_size.height,
    };

    draw_icon_face(
        &mut output,
        &sampler,
        face_layers[1],
        [left, middle, middle_down, left_down],
        BLOCK_ICON_LEFT_BRIGHTNESS,
    );
    draw_icon_face(
        &mut output,
        &sampler,
        face_layers[2],
        [right, middle, middle_down, right_down],
        BLOCK_ICON_RIGHT_BRIGHTNESS,
    );
    draw_icon_face(
        &mut output,
        &sampler,
        face_layers[0],
        [top, right, middle, left],
        BLOCK_ICON_TOP_BRIGHTNESS,
    );

    output
}

/// 方块 atlas 采样视图。
struct AtlasSampler<'a> {
    /// atlas 像素数据，格式固定为 RGBA8。
    data: &'a [u8],
    /// atlas 宽度。
    width: u32,
    /// atlas 高度。
    height: u32,
}

/// 绘制一个平行四边形面，并把局部坐标映射回 16x16 方块贴图。
fn draw_icon_face(
    output: &mut [u8],
    sampler: &AtlasSampler<'_>,
    layer: u32,
    corners: [Vec2; 4],
    brightness: f32,
) {
    let min_x = corners
        .iter()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_x = corners
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(ITEM_MODEL_PREVIEW_SIZE as f32 - 1.0) as u32;
    let min_y = corners
        .iter()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .floor()
        .max(0.0) as u32;
    let max_y = corners
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        .min(ITEM_MODEL_PREVIEW_SIZE as f32 - 1.0) as u32;

    let origin = corners[0];
    let u_axis = corners[1] - origin;
    let v_axis = corners[3] - origin;
    let determinant = u_axis.x * v_axis.y - u_axis.y * v_axis.x;
    if determinant.abs() <= f32::EPSILON {
        return;
    }

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let local = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) - origin;
            let u = (local.x * v_axis.y - local.y * v_axis.x) / determinant;
            let v = (u_axis.x * local.y - u_axis.y * local.x) / determinant;
            if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
                continue;
            }

            let Some(sample) = sample_block_atlas(sampler, layer, u, v) else {
                continue;
            };
            blend_icon_pixel(output, x, y, sample, brightness);
        }
    }
}

/// 从方块 atlas 的指定层采样一个像素。
fn sample_block_atlas(sampler: &AtlasSampler<'_>, layer: u32, u: f32, v: f32) -> Option<[u8; 4]> {
    let tile_size = TILE_SIZE;
    let atlas_x = (u.clamp(0.0, 0.999) * tile_size as f32) as u32;
    let atlas_y =
        layer * CHUNK_SIZE as u32 * tile_size + (v.clamp(0.0, 0.999) * tile_size as f32) as u32;
    if atlas_x >= sampler.width || atlas_y >= sampler.height {
        return None;
    }

    let index = ((atlas_y * sampler.width + atlas_x) * 4) as usize;
    let pixel = sampler.data.get(index..index + 4)?;
    Some([pixel[0], pixel[1], pixel[2], pixel[3]])
}

/// 把采样像素写入输出图标，保留透明度并应用固定面亮度。
fn blend_icon_pixel(output: &mut [u8], x: u32, y: u32, sample: [u8; 4], brightness: f32) {
    if sample[3] == 0 {
        return;
    }

    let index = ((y * ITEM_MODEL_PREVIEW_SIZE + x) * 4) as usize;
    let Some(pixel) = output.get_mut(index..index + 4) else {
        return;
    };

    pixel[0] = ((sample[0] as f32 * brightness).round()).clamp(0.0, 255.0) as u8;
    pixel[1] = ((sample[1] as f32 * brightness).round()).clamp(0.0, 255.0) as u8;
    pixel[2] = ((sample[2] as f32 * brightness).round()).clamp(0.0, 255.0) as u8;
    pixel[3] = sample[3];
}

/// 为 GUI 图标创建明亮且稳定的预览材质。
///
/// 复杂模型的离屏预览不再依赖场景灯光；方块图标则直接走 CPU 亮度合成。
fn preview_material(
    source: &Handle<StandardMaterial>,
    materials: &mut Assets<StandardMaterial>,
) -> Handle<StandardMaterial> {
    let Some(source_material) = materials.get(source) else {
        return source.clone();
    };

    let mut material = source_material.clone();
    material.base_color = Color::WHITE;
    material.unlit = true;
    material.perceptual_roughness = 1.0;
    material.cull_mode = None;
    materials.add(material)
}
