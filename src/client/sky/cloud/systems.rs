//! 创建云场实体，并在渲染帧推进体积云着色参数与近景云片表现。
//!
//! 云层用「天空球体 mesh + raymarching 扩展材质」：球体仅作为触发片元着色的
//! 载体，云的真实形状由 `cloud_volume.wgsl` 内的体素密度场决定。

use super::components::{CloudLayer, CloudPatch, CloudWeatherState};
use super::material::{CloudVolumeExtension, CloudVolumeMaterial, CloudVolumeUniform};
use super::texture::{cloud_image_to_bevy, generate_cloud_texture};
use crate::client::camera::FpsCamera;
use crate::client::presentation::TimeOfDay;
use crate::client::sky::components::Sun;
use crate::content::cloud::CloudRegistry;
use bevy::ecs::system::SystemParam;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::math::Vec4;
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};

/// 云层垂直厚度的一半（世界单位），决定云体的高度范围（云厚 8 米）。
// ARTShade 使用 8 个世界单位的薄云层；三层之间的间隔由着色器固定为 48。
const CLOUD_HALF_THICKNESS: f32 = 4.0;

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
    cloud_materials: ResMut<'w, Assets<CloudVolumeMaterial>>,
    camera_query: Query<'w, 's, &'static GlobalTransform, With<FpsCamera>>,
}

/// 在进入游戏且内容注册表就绪后创建云场实体。
///
/// 云层用「天空球体 mesh + raymarching 扩展材质」：球体仅作为触发片元着色的
/// 载体，云的真实形状由 shader 内的二维覆盖场和垂直边缘淡出决定。
/// 近景云片仍保留 billboard 贴图，用于补充云场纵深。
pub fn setup_cloud_system(mut commands: Commands, mut params: CloudSetupParams) {
    cleanup_cloud_entities(&mut commands, &mut params.runtime);
    let Some(definition) = params.cloud_registry.primary() else {
        log::warn!("[云层] 未加载云层定义，跳过云场构建");
        return;
    };

    let weather = params.weather.normalized();
    let camera_transform = params.camera_query.iter().next();
    let camera_pos = camera_transform
        .map(|transform| transform.translation())
        .unwrap_or(Vec3::ZERO);
    let camera_rotation = camera_transform
        .map(GlobalTransform::rotation)
        .unwrap_or(Quat::IDENTITY);
    let camera_forward = camera_transform
        .map(|transform| transform.forward().as_vec3())
        .unwrap_or(-Vec3::Z);

    // 体积云层：天空球体 mesh + raymarching 扩展材质。
    if let Some(layer) = definition.layers.first() {
        // 使用朝向相机的全屏代理平面，模拟 ARTShade 的后处理云 pass。
        let proxy_distance = 256.0;
        let wind = Vec2::from_array(layer.wind_direction).normalize_or_zero();
        let material = params.cloud_materials.add(CloudVolumeMaterial {
            base: StandardMaterial {
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                cull_mode: None,
                ..default()
            },
            extension: CloudVolumeExtension {
                uniform: CloudVolumeUniform {
                    coverage: weather.coverage,
                    cloud_min_y: layer.height - CLOUD_HALF_THICKNESS,
                    cloud_max_y: layer.height + CLOUD_HALF_THICKNESS,
                    wind_speed: layer.speed,
                    wind_direction: Vec4::new(wind.x, wind.y, 0.0, 0.0),
                    density_threshold: definition.density.clamp(0.05, 0.95),
                    visibility: weather.visibility,
                    tint_day: Vec4::new(
                        layer.tint_day[0],
                        layer.tint_day[1],
                        layer.tint_day[2],
                        layer.opacity,
                    ),
                    tint_night: Vec4::new(
                        layer.tint_night[0],
                        layer.tint_night[1],
                        layer.tint_night[2],
                        layer.opacity,
                    ),
                    tint_sunset: Vec4::new(
                        layer.tint_sunset[0],
                        layer.tint_sunset[1],
                        layer.tint_sunset[2],
                        layer.opacity,
                    ),
                    ..default()
                },
            },
        });
        let mesh = params
            .meshes
            .add(Mesh::from(Rectangle::new(2048.0, 2048.0)));
        let entity = commands
            .spawn((
                CloudLayer {
                    definition: layer.clone(),
                },
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::from_translation(camera_pos + camera_forward * proxy_distance)
                    .with_rotation(camera_rotation),
                Visibility::default(),
                NotShadowCaster,
                NotShadowReceiver,
            ))
            .id();
        params.runtime.layer_entities.push(entity);
    }

    // 近景 billboard 云片：保留贴图方案，补充云场纵深。
    if definition.patches.enabled && definition.patches.count > 0 {
        let cloud_image = params
            .images
            .add(cloud_image_to_bevy(generate_cloud_texture(
                definition.density,
                definition.seed,
            )));
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
        "[云层] 已生成 {} 个体积云层与 {} 个近景云片",
        params.runtime.layer_entities.len(),
        params.runtime.patch_entities.len()
    );
}

/// 每帧让云层球体跟随相机，并把昼夜、天气、时间与相机位置写入云材质 uniform。
#[allow(clippy::type_complexity)]
pub fn cloud_volume_update_system(
    time: Res<Time>,
    time_of_day: Res<TimeOfDay>,
    weather: Res<CloudWeatherState>,
    camera_query: Query<&GlobalTransform, With<FpsCamera>>,
    sun_query: Query<&GlobalTransform, (With<Sun>, Without<FpsCamera>)>,
    mut cloud_query: Query<(
        &CloudLayer,
        &MeshMaterial3d<CloudVolumeMaterial>,
        &mut Transform,
    )>,
    mut materials: ResMut<Assets<CloudVolumeMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();
    let weather = weather.normalized();
    let night = time_of_day.night_factor();
    let twilight = time_of_day.twilight_factor();
    let twilight_glow = (4.0 * twilight * (1.0 - twilight)).clamp(0.0, 1.0);
    let elapsed = time.elapsed_secs();
    let sun_direction = sun_query
        .iter()
        .next()
        .map(|transform| -transform.forward().as_vec3())
        .unwrap_or_else(|| CloudVolumeUniform::default().sun_direction.truncate())
        .normalize_or_zero();

    for (layer, material_handle, mut transform) in &mut cloud_query {
        // 代理平面跟随相机姿态；云层真实高度由 uniform 的 slab 范围决定。
        transform.translation = camera_pos + camera_transform.forward().as_vec3() * 256.0;
        transform.rotation = camera_transform.rotation();
        if let Some(mut material) = materials.get_mut(&material_handle.0) {
            material.extension.uniform.time_seconds = elapsed;
            material.extension.uniform.coverage = weather.coverage;
            material.extension.uniform.wind_speed =
                layer.definition.speed * weather.wind_multiplier;
            material.extension.uniform.night_factor = night;
            material.extension.uniform.twilight_glow = twilight_glow;
            material.extension.uniform.visibility = weather.visibility;
            material.extension.uniform.sun_direction =
                Vec4::new(sun_direction.x, sun_direction.y, sun_direction.z, 0.0);
            material.extension.uniform.camera_position =
                Vec4::new(camera_pos.x, camera_pos.y, camera_pos.z, 0.0);
        }
    }
}

fn visible_opacity(opacity: f32, weather: CloudWeatherState) -> f32 {
    opacity.clamp(0.0, 1.0) * weather.coverage * weather.visibility
}

/// 每帧根据昼夜和天气状态更新近景云片材质。
pub fn cloud_tint_system(
    time_of_day: Res<TimeOfDay>,
    weather: Res<CloudWeatherState>,
    runtime: Res<CloudRuntime>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let night = time_of_day.night_factor();
    let weather = weather.normalized();

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
