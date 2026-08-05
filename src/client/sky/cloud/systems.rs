//! 创建云场实体，并在渲染帧推进世界连续漂移与昼夜染色。

use super::components::{CloudLayer, CloudPatch, CloudWeatherState};

use super::texture::{cloud_image_to_bevy, generate_cloud_texture};
use crate::client::camera::FpsCamera;
use crate::client::presentation::TimeOfDay;
use crate::content::cloud::CloudRegistry;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::Affine2;
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};

/// 云场运行时资源，集中管理实体生命周期和近景云片共享材质。
#[derive(Resource, Default)]
pub struct CloudRuntime {
    /// 已创建的云层实体集合。
    layer_entities: Vec<Entity>,
    /// 已创建的云片实体集合。
    patch_entities: Vec<Entity>,
    /// 云片共享材质句柄。
    patch_material: Option<Handle<StandardMaterial>>,
    /// 近景云片定义的不透明度，天气变化不会反复乘入该基准。
    patch_opacity: f32,
}

/// 离开世界时清理云实体，避免下一个世界沿用旧云场。
pub fn cleanup_cloud_system(mut commands: Commands, mut runtime: ResMut<CloudRuntime>) {
    cleanup_cloud_entities(&mut commands, &mut runtime);
}

fn cleanup_cloud_entities(commands: &mut Commands, runtime: &mut CloudRuntime) {
    for entity in runtime.layer_entities.iter().chain(&runtime.patch_entities) {
        commands.entity(*entity).despawn();
    }
    runtime.layer_entities.clear();
    runtime.patch_entities.clear();
    runtime.patch_material = None;
    runtime.patch_opacity = 0.0;
}

/// 聚合云场创建阶段所需的内容、资源和相机查询。
#[derive(SystemParam)]
pub(super) struct CloudSetupParams<'w, 's> {
    runtime: ResMut<'w, CloudRuntime>,
    cloud_registry: Res<'w, CloudRegistry>,
    weather: Res<'w, CloudWeatherState>,
    meshes: ResMut<'w, Assets<Mesh>>,
    images: ResMut<'w, Assets<Image>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    camera_query: Query<'w, 's, &'static GlobalTransform, With<FpsCamera>>,
}

/// 在进入游戏且内容注册表就绪后创建云场实体。
///
/// 云平面跟随相机只负责保持视野覆盖；真实世界连续性由 UV 相位补偿相机位移，
/// 因而不会在区块边界或玩家长距离移动时出现可见跳变。
pub fn setup_cloud_system(mut commands: Commands, mut params: CloudSetupParams) {
    cleanup_cloud_entities(&mut commands, &mut params.runtime);
    let Some(definition) = params.cloud_registry.primary() else {
        log::warn!("[Cloud] no cloud definition loaded; skipping cloud setup");
        return;
    };

    let weather = params.weather.normalized();
    let camera_pos = params
        .camera_query
        .iter()
        .next()
        .map(|transform| transform.translation())
        .unwrap_or(Vec3::ZERO);
    let camera_xz = Vec2::new(camera_pos.x, camera_pos.z);
    let cloud_image = params
        .images
        .add(cloud_image_to_bevy(generate_cloud_texture(
            definition.density,
            definition.seed,
        )));

    for layer in &definition.layers {
        let size = layer.size.max(1.0);
        let uv_offset = world_uv_offset(camera_xz, size);
        let material = params.materials.add(StandardMaterial {
            base_color_texture: Some(cloud_image.clone()),
            base_color: Color::srgba(
                layer.tint_day[0],
                layer.tint_day[1],
                layer.tint_day[2],
                visible_opacity(layer.opacity, weather),
            ),
            uv_transform: Affine2::from_translation(uv_offset),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        let mesh = params.meshes.add(Mesh::from(Rectangle::new(size, size)));
        let entity = commands
            .spawn((
                CloudLayer {
                    definition: layer.clone(),
                    uv_offset,
                    last_camera_position: camera_xz,
                },
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_xyz(camera_pos.x, layer.height, camera_pos.z)
                    .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                Visibility::default(),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        params.runtime.layer_entities.push(entity);
    }

    if definition.patches.enabled && definition.patches.count > 0 {
        let patch_material = params.materials.add(StandardMaterial {
            base_color_texture: Some(cloud_image),
            base_color: Color::srgba(
                1.0,
                1.0,
                1.0,
                visible_opacity(definition.patches.opacity, weather),
            ),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        let patch_mesh = params.meshes.add(Mesh::from(Rectangle::new(1.0, 1.0)));
        let mut rng = rand::rngs::StdRng::seed_from_u64(definition.seed as u64);
        let radius = definition.patches.spawn_radius.max(1.0);
        let scale_min = definition.patches.scale_min.max(1.0);
        let scale_max = definition.patches.scale_max.max(scale_min);
        let height = definition
            .layers
            .first()
            .map_or(128.0, |layer| layer.height);

        for _ in 0..definition.patches.count.min(128) {
            let angle = rng.random_range(0.0..std::f32::consts::TAU);
            let distance = rng.random_range(0.3..1.0) * radius;
            let scale = rng.random_range(scale_min..=scale_max);
            let height_offset = rng.random_range(-8.0..8.0);
            let position = Vec3::new(
                camera_pos.x + angle.cos() * distance,
                height + height_offset,
                camera_pos.z + angle.sin() * distance,
            );
            let entity = commands
                .spawn((
                    CloudPatch { scale, radius },
                    Mesh3d(patch_mesh.clone()),
                    MeshMaterial3d(patch_material.clone()),
                    Transform::from_translation(position).with_scale(Vec3::new(scale, scale, 1.0)),
                    Visibility::default(),
                    NotShadowCaster,
                    NotShadowReceiver,
                ))
                .id();
            params.runtime.patch_entities.push(entity);
        }
        params.runtime.patch_material = Some(patch_material);
        params.runtime.patch_opacity = definition.patches.opacity;
    }

    log::info!(
        "[Cloud] spawned {} layer(s) and {} patch(es)",
        params.runtime.layer_entities.len(),
        params.runtime.patch_entities.len()
    );
}

/// 每帧推进云层漂移，并用相机位移补偿保持云场的世界连续性。
#[allow(clippy::type_complexity)]
pub fn cloud_drift_system(
    time: Res<Time>,
    camera_query: Query<&GlobalTransform, With<FpsCamera>>,
    weather: Res<CloudWeatherState>,
    mut cloud_query: Query<(
        &mut CloudLayer,
        &MeshMaterial3d<StandardMaterial>,
        &mut Transform,
        &mut Visibility,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();
    let camera_xz = Vec2::new(camera_pos.x, camera_pos.z);
    let weather = weather.normalized();
    let delta = time.delta_secs().min(0.1);

    for (mut layer, material, mut transform, mut visibility) in &mut cloud_query {
        let size = layer.definition.size.max(1.0);
        let camera_delta = camera_xz - layer.last_camera_position;
        let direction = Vec2::from_array(layer.definition.wind_direction).normalize_or_zero();
        let wind_delta = direction * layer.definition.speed * weather.wind_multiplier * delta;
        let offset_delta = camera_delta / size - wind_delta / size;
        layer.uv_offset = Vec2::new(
            advance_uv_offset(layer.uv_offset.x, offset_delta.x, 1.0),
            advance_uv_offset(layer.uv_offset.y, offset_delta.y, 1.0),
        );
        layer.last_camera_position = camera_xz;
        transform.translation = Vec3::new(camera_pos.x, layer.definition.height, camera_pos.z);

        if let Some(mut material_asset) = materials.get_mut(&material.0) {
            material_asset.uv_transform = Affine2::from_translation(layer.uv_offset);
        }
        *visibility = Visibility::Visible;
    }
}

/// 计算云层昼夜和黄昏叠加后的 RGB 色调。
pub fn cloud_tint_color(
    tint_day: [f32; 3],
    tint_night: [f32; 3],
    tint_sunset: [f32; 3],
    night: f32,
    twilight_glow: f32,
) -> [f32; 3] {
    let night = night.clamp(0.0, 1.0);
    let twilight_glow = twilight_glow.clamp(0.0, 1.0);
    let mut tint = [
        tint_day[0] + (tint_night[0] - tint_day[0]) * night,
        tint_day[1] + (tint_night[1] - tint_day[1]) * night,
        tint_day[2] + (tint_night[2] - tint_day[2]) * night,
    ];
    for (value, sunset) in tint.iter_mut().zip(&tint_sunset) {
        *value += (sunset - *value) * twilight_glow * 0.5;
    }
    tint.map(|value| value.clamp(0.0, 1.0))
}

/// 保留标量 UV 相位工具，供内容与表现层测试验证回绕边界。
pub fn advance_uv_offset(current: f32, speed: f32, delta_secs: f32) -> f32 {
    (current + speed * delta_secs).rem_euclid(1.0)
}

fn world_uv_offset(camera_xz: Vec2, size: f32) -> Vec2 {
    wrap_uv_offset(camera_xz / size.max(1.0))
}

fn wrap_uv_offset(offset: Vec2) -> Vec2 {
    Vec2::new(offset.x.rem_euclid(1.0), offset.y.rem_euclid(1.0))
}

fn visible_opacity(opacity: f32, weather: CloudWeatherState) -> f32 {
    opacity.clamp(0.0, 1.0) * weather.coverage * weather.visibility
}

/// 每帧根据昼夜和天气状态更新云层及近景云片材质。
#[allow(clippy::type_complexity)]
pub fn cloud_tint_system(
    time_of_day: Res<TimeOfDay>,
    weather: Res<CloudWeatherState>,
    cloud_query: Query<(&CloudLayer, &MeshMaterial3d<StandardMaterial>)>,
    runtime: Res<CloudRuntime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let night = time_of_day.night_factor();
    let twilight = time_of_day.twilight_factor();
    let twilight_glow = (4.0_f32 * twilight * (1.0 - twilight)).clamp(0.0, 1.0);
    let weather = weather.normalized();

    for (layer, material) in &cloud_query {
        let Some(mut material_asset) = materials.get_mut(&material.0) else {
            continue;
        };
        let tint = cloud_tint_color(
            layer.definition.tint_day,
            layer.definition.tint_night,
            layer.definition.tint_sunset,
            night,
            twilight_glow,
        );
        material_asset.base_color = Color::srgba(
            tint[0],
            tint[1],
            tint[2],
            visible_opacity(layer.definition.opacity, weather),
        );
    }

    if let Some(handle) = &runtime.patch_material
        && let Some(mut material_asset) = materials.get_mut(handle)
    {
        material_asset.base_color = Color::srgba(
            1.0 - night * 0.75,
            1.0 - night * 0.73,
            1.0 - night * 0.62,
            visible_opacity(runtime.patch_opacity, weather),
        );
    }
}

/// 让近景云片朝向相机，并在离开环带后从对侧重生。
#[allow(clippy::type_complexity)]
pub fn cloud_patch_system(
    camera_query: Query<&GlobalTransform, With<FpsCamera>>,
    mut patch_query: Query<(&CloudPatch, &mut Transform)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();

    for (patch, mut transform) in &mut patch_query {
        let dir_to_camera = (camera_pos - transform.translation).normalize_or_zero();
        if dir_to_camera.length_squared() > 0.001 {
            transform.rotation = Quat::from_rotation_arc(Vec3::Z, dir_to_camera);
        }

        let offset = transform.translation - camera_pos;
        let horizontal = Vec2::new(offset.x, offset.z).length();
        if horizontal > patch.radius * 2.0 {
            let direction = Vec2::new(offset.x, offset.z).normalize_or_zero() * patch.radius;
            transform.translation = Vec3::new(
                camera_pos.x - direction.x,
                transform.translation.y,
                camera_pos.z - direction.y,
            );
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/client/sky/cloud/systems.rs"]
mod tests;
