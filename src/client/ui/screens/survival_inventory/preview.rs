//! 管理生存物品栏中的离屏玩家模型预览。

use bevy::camera::{RenderTarget, ScalingMode, visibility::RenderLayers};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::client::player::model::config::PlayerModelConfig;
use crate::client::ui::components::SurvivalPlayerPreviewCamera;

const PREVIEW_LAYER: usize = 7;
/// 创建只由生存物品栏使用的离屏玩家预览。
pub(super) fn spawn_player_preview(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &PlayerModelConfig,
) -> Handle<Image> {
    let image = Image::new_target_texture(384, 320, TextureFormat::Rgba8UnormSrgb, None);
    let image_handle = images.add(image);
    let target = Vec3::new(0.0, -750.0, 0.0);
    let preview_layer = RenderLayers::layer(PREVIEW_LAYER);
    let (root, rig) =
        crate::client::player::model::rig::spawn_player_rig(commands, meshes, materials, config);

    commands.entity(root).insert((
        Transform {
            translation: target,
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            ..default()
        },
        preview_layer.clone(),
        Name::new("InventoryPlayerPreview"),
    ));
    for entity in rig.mesh_entities {
        commands
            .entity(entity)
            .insert((preview_layer.clone(), NotShadowCaster));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(target + Vec3::new(3.0, 4.0, 4.0)).looking_at(target, Vec3::Y),
        preview_layer.clone(),
        Name::new("InventoryPreviewLight"),
    ));

    commands.spawn((
        SurvivalPlayerPreviewCamera,
        Camera3d::default(),
        Camera {
            order: -8,
            is_active: false,
            clear_color: Color::NONE.into(),
            ..default()
        },
        RenderTarget::Image(image_handle.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 2.8,
                height: 2.6,
            },
            near: 0.0,
            far: 32.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(target + Vec3::new(2.8, 1.35, 4.6))
            .looking_at(target + Vec3::Y * 0.12, Vec3::Y),
        preview_layer,
        Name::new("InventoryPreviewCamera"),
    ));

    image_handle
}
