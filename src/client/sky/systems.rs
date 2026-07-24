use super::constants::*;
use crate::app::flow::GameSettings;
use crate::client::sky::components::*;
use crate::client::sky::texture;
use crate::content::constant::world::CHUNK_SIZE;
use crate::game::world::time::TimeOfDay;
use bevy::camera::{Exposure, visibility::RenderLayers};
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{
    Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, GlobalAmbientLight,
    NotShadowCaster, NotShadowReceiver, VolumetricFog, VolumetricLight,
};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use std::f32::consts::TAU;

pub fn setup_sky_system(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 生成世界大气
    let earth_medium = scattering_mediums.add(ScatteringMedium::default());
    commands.spawn((Atmosphere::earth(earth_medium),));

    // 构造级联阴影
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        // 第一个阴影级联的远边界
        first_cascade_far_bound: 18.0,
        // 阴影的最大渲染距离
        maximum_distance: 112.0,
        // 级联数量
        num_cascades: 4,
        overlap_proportion: 0.28,
        ..default()
    }
    .build();

    // 生成太阳光
    commands.spawn((
        Sun,
        DirectionalLight {
            illuminance: DAY_SUN_ILLUMINANCE,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
        RenderLayers::layer(0).with(1),
        // 使用体积光计算光线
        VolumetricLight,
        cascade_shadow_config.clone(),
    ));

    // 生成月光
    commands.spawn((
        Moon,
        DirectionalLight {
            color: Color::srgb(0.8, 0.85, 1.0),
            illuminance: light_consts::lux::FULL_MOON_NIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
        RenderLayers::layer(0).with(1),
        cascade_shadow_config,
    ));

    // 生成太阳纹理
    let sun_texture = texture::generate_sun_texture(SUN_TEXTURE_SIZE);
    let sun_image = images.add(texture::rgba_image_to_bevy(sun_texture));
    let sun_material = materials.add(StandardMaterial {
        base_color_texture: Some(sun_image),
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let sun_quad = meshes.add(Rectangle::new(CELESTIAL_MESH_SIZE, CELESTIAL_MESH_SIZE));

    commands.spawn((
        SunMesh,
        Mesh3d(sun_quad),
        MeshMaterial3d(sun_material),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 生成月亮纹理
    let moon_texture = texture::generate_moon_texture(MOON_TEXTURE_SIZE);
    let moon_image = images.add(texture::rgba_image_to_bevy(moon_texture));
    let moon_material = materials.add(StandardMaterial {
        base_color_texture: Some(moon_image),
        base_color: Color::srgb(0.9, 0.92, 1.0),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    let moon_quad = meshes.add(Rectangle::new(CELESTIAL_MESH_SIZE, CELESTIAL_MESH_SIZE));

    commands.spawn((
        MoonMesh,
        Mesh3d(moon_quad),
        MeshMaterial3d(moon_material),
        Transform::default(),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    // 生成星空
    let star_texture = texture::generate_star_texture(STAR_TEXTURE_SIZE);
    let star_image = images.add(texture::rgba_image_to_bevy(star_texture));

    // 使用随机种子保证每次启动星空一致
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);

    for _ in 0..STAR_COUNT {
        // 在球面上均匀分布
        let theta: f32 = rng.random_range(0.0..TAU);
        let phi: f32 = rng.random_range(0.0..std::f32::consts::PI);
        let star_dir = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());

        // 星星亮度随机（0.3 ~ 1.0）
        let brightness: f32 = rng.random_range(0.3..1.0);

        let star_material = materials.add(StandardMaterial {
            base_color_texture: Some(star_image.clone()),
            base_color: Color::srgba(brightness, brightness, brightness * 1.05, brightness),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });

        let star_quad = meshes.add(Rectangle::new(STAR_QUAD_SIZE, STAR_QUAD_SIZE));

        let star_pos = star_dir * STAR_SPHERE_RADIUS;

        commands.spawn((
            Stars,
            Mesh3d(star_quad),
            MeshMaterial3d(star_material),
            Transform::from_translation(star_pos).looking_to(-star_dir, Vec3::Y),
        ));
    }
}

pub fn atmosphere_system(
    time_of_day: Res<TimeOfDay>,
    settings: Res<GameSettings>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), (With<Sun>, Without<Moon>)>,
    mut moon_query: Query<(&mut Transform, &mut DirectionalLight), (With<Moon>, Without<Sun>)>,
    mut camera_query: Query<
        (
            &mut Exposure,
            Option<&mut VolumetricFog>,
            Option<&mut DistanceFog>,
            Option<&mut AtmosphereEnvironmentMapLight>,
        ),
        With<crate::shared::components::FpsCamera>,
    >,
) {
    // 太阳当前弧度角 (0.0 到 2π)
    let sun_angle = ((time_of_day.current_time + 6.0) / 24.0) * TAU;
    // 月亮与太阳永远保持 180 度正对立
    let moon_angle = sun_angle + std::f32::consts::PI;

    let mut current_sun_y = 0.0;
    let mut sun_fade = 0.0;

    if let Ok((mut sun_transform, mut sun_light)) = sun_query.single_mut() {
        sun_transform.translation = Vec3::ZERO;
        sun_transform.rotation = Quat::from_rotation_x(sun_angle);

        let sun_forward_y = sun_transform.forward().y;
        current_sun_y = sun_forward_y;

        // 太阳高度淡出
        sun_fade = ((-sun_forward_y + 0.12) / 1.12).clamp(0.0, 1.0);
        sun_light.illuminance = sun_fade * DAY_SUN_ILLUMINANCE;

        // 日出/日落时太阳光颜色偏暖
        let twilight = time_of_day.twilight_factor();
        if twilight > 0.0 && twilight < 1.0 {
            // 过渡期：混合暖色
            let warmth = 1.0 - (twilight - 0.5).abs() * 2.0; // 在中间最暖
            sun_light.color = Color::srgb(1.0, 0.98 - warmth * 0.16, 0.94 - warmth * 0.30);
        } else {
            sun_light.color = Color::srgb(1.0, 0.99, 0.97);
        }
    }

    if let Ok((mut moon_transform, mut moon_light)) = moon_query.single_mut() {
        moon_transform.translation = Vec3::ZERO;
        moon_transform.rotation = Quat::from_rotation_x(moon_angle);

        let moon_forward_y = moon_transform.forward().y;
        let moon_fade = ((-moon_forward_y + 0.12) / 1.12).clamp(0.0, 1.0);
        moon_light.illuminance =
            MIN_MOON_ILLUMINANCE + moon_fade * (MAX_MOON_ILLUMINANCE - MIN_MOON_ILLUMINANCE);
    }

    let night_factor = time_of_day.night_factor();
    let night_mix = smoothstep(night_factor);

    // 深夜保留冷色环境光，让地形仍可辨认，同时避免看起来像白天。
    let day_mix = 1.0 - night_mix;
    ambient_light.color = Color::srgb(
        0.30 + (0.78 - 0.30) * day_mix,
        0.40 + (0.87 - 0.40) * day_mix,
        0.68 + (1.00 - 0.68) * day_mix,
    );
    ambient_light.brightness =
        DAY_AMBIENT_BRIGHTNESS + (NIGHT_AMBIENT_BRIGHTNESS - DAY_AMBIENT_BRIGHTNESS) * night_mix;

    let twilight = time_of_day.twilight_factor();
    let twilight_glow = (4.0 * twilight * (1.0 - twilight)).clamp(0.0, 1.0);
    let view_distance = settings.render_distance.max(4) as f32 * CHUNK_SIZE as f32;
    let fog_start = (view_distance * 0.48).clamp(52.0, 180.0);
    let fog_end = (view_distance * 1.45).clamp(160.0, 560.0);

    for (mut exposure, volumetric_fog, distance_fog, environment_light) in &mut camera_query {
        exposure.ev100 = visibility_exposure_ev100(sun_fade, current_sun_y, night_factor);

        // 体积雾环境光联动
        if let Some(mut vol_fog) = volumetric_fog {
            vol_fog.ambient_color = Color::srgb(
                0.25 + (0.62 - 0.25) * day_mix,
                0.34 + (0.73 - 0.34) * day_mix,
                0.58 + (0.84 - 0.58) * day_mix,
            );
            if twilight > 0.0 && twilight < 1.0 {
                vol_fog.ambient_intensity = TWILIGHT_FOG_AMBIENT;
            } else if night_factor > 0.5 {
                vol_fog.ambient_intensity = NIGHT_FOG_AMBIENT;
            } else {
                vol_fog.ambient_intensity = DAY_FOG_AMBIENT;
            }
        }
        if let Some(mut fog) = distance_fog {
            let base = [
                0.10 + (0.57 - 0.10) * day_mix,
                0.15 + (0.69 - 0.15) * day_mix,
                0.28 + (0.79 - 0.28) * day_mix,
            ];
            let warm = [0.78, 0.58, 0.43];
            let warmth = twilight_glow * 0.34;
            fog.color = Color::srgba(
                base[0] + (warm[0] - base[0]) * warmth,
                base[1] + (warm[1] - base[1]) * warmth,
                base[2] + (warm[2] - base[2]) * warmth,
                0.64 + (0.50 - 0.64) * day_mix,
            );
            fog.directional_light_color = Color::srgba(1.0, 0.76, 0.48, 0.10 + 0.18 * day_mix);
            fog.directional_light_exponent = 24.0;
            fog.falloff = FogFalloff::Linear {
                start: fog_start,
                end: fog_end,
            };
        }
        if let Some(mut environment_light) = environment_light {
            environment_light.intensity = 0.70 + 1.10 * day_mix;
        }
    }
}

fn visibility_exposure_ev100(sun_fade: f32, sun_y: f32, night_factor: f32) -> f32 {
    let sun_height = (-sun_y).clamp(0.0, 1.0);
    let daylight_ev100 = 11.8 + 2.3 * sun_height;
    let twilight_ev100 = NIGHT_EXPOSURE_EV100 + (1.0 - night_factor.clamp(0.0, 1.0)) * 1.8;
    let daylight_mix = smoothstep((sun_fade / 0.25).clamp(0.0, 1.0));
    twilight_ev100 + (daylight_ev100 - twilight_ev100) * daylight_mix
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

/// 天体纹理处理系统
pub fn celestial_mesh_system(
    camera_query: Query<&GlobalTransform, With<crate::shared::components::FpsCamera>>,
    mut sun_mesh_query: Query<&mut Transform, (With<SunMesh>, Without<MoonMesh>)>,
    mut moon_mesh_query: Query<&mut Transform, (With<MoonMesh>, Without<SunMesh>)>,
    sun_query: Query<&Transform, (With<Sun>, Without<SunMesh>, Without<MoonMesh>)>,
    moon_query: Query<&Transform, (With<Moon>, Without<SunMesh>, Without<MoonMesh>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let camera_pos = camera_transform.translation();

    // 太阳方向
    if let (Ok(mut sun_mesh_transform), Ok(sun_light_transform)) =
        (sun_mesh_query.single_mut(), sun_query.single())
    {
        let sun_source_dir = -sun_light_transform.forward();
        let sun_pos = camera_pos + sun_source_dir * CELESTIAL_DISTANCE;

        let dir_to_camera = (camera_pos - sun_pos).normalize();
        sun_mesh_transform.translation = sun_pos;
        sun_mesh_transform.rotation = Quat::from_rotation_arc(Vec3::Z, dir_to_camera);

        let above_horizon = sun_source_dir.y + 0.15;
        sun_mesh_transform.scale = Vec3::splat(if above_horizon > 0.0 { 1.0 } else { 0.0 });
    }

    // 月亮方向
    if let (Ok(mut moon_mesh_transform), Ok(moon_light_transform)) =
        (moon_mesh_query.single_mut(), moon_query.single())
    {
        let moon_source_dir = -moon_light_transform.forward();
        let moon_pos = camera_pos + moon_source_dir * CELESTIAL_DISTANCE;

        let dir_to_camera = (camera_pos - moon_pos).normalize();
        moon_mesh_transform.translation = moon_pos;
        moon_mesh_transform.rotation = Quat::from_rotation_arc(Vec3::Z, dir_to_camera);

        let above_horizon = moon_source_dir.y + 0.15;
        moon_mesh_transform.scale = Vec3::splat(if above_horizon > 0.0 { 1.0 } else { 0.0 });
    }
}

/// 星空可见性系统
pub fn stars_visibility_system(
    time_of_day: Res<TimeOfDay>,
    mut star_query: Query<&mut Visibility, With<Stars>>,
) {
    let night_factor = time_of_day.night_factor();

    // 夜晚因子 > 0.3 时开始显示星星
    let visible = night_factor > 0.3;

    for mut visibility in &mut star_query {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/client/sky/systems.rs"]
mod tests;
