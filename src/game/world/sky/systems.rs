use std::f32::consts::TAU;
use crate::game::world::sky::components::*;
use crate::game::world::time::TimeOfDay;
use bevy::camera::Exposure;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{Atmosphere, CascadeShadowConfigBuilder, NotShadowCaster, NotShadowReceiver, VolumetricFog, VolumetricLight};
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use crate::engine::constant::sky::*;
use crate::game::world::sky::texture;

pub fn setup_sky_system(
    mut commands: Commands,
    mut scattering_mediums: ResMut<Assets<ScatteringMedium>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 生成世界大气
    let earth_medium = scattering_mediums.add(ScatteringMedium::default());
    commands.spawn((
        Atmosphere::earth(earth_medium),
    ));

    // 构造级联阴影
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        // 第一个阴影级联的远边界
        first_cascade_far_bound: 16.0,
        // 阴影的最大渲染距离
        maximum_distance: 64.0,
        // 级联数量
        num_cascades: 4,
        ..default()
    }
        .build();

    // 生成太阳光
    commands.spawn((
        Sun,
        DirectionalLight{
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
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
        let star_dir = Vec3::new(
            phi.sin() * theta.cos(),
            phi.cos(),
            phi.sin() * theta.sin(),
        );

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
            Transform::from_translation(star_pos)
                .looking_to(-star_dir, Vec3::Y),
        ));
    }
}

pub fn atmosphere_system(
    time_of_day: Res<TimeOfDay>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), (With<Sun>, Without<Moon>)>,
    mut moon_query: Query<(&mut Transform, &mut DirectionalLight), (With<Moon>, Without<Sun>)>,
    mut camera_query: Query<(&mut Exposure, Option<&mut VolumetricFog>), With<Camera3d>>,
){
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
        sun_light.illuminance = sun_fade * light_consts::lux::RAW_SUNLIGHT;

        // 日出/日落时太阳光颜色偏暖
        let twilight = time_of_day.twilight_factor();
        if twilight > 0.0 && twilight < 1.0 {
            // 过渡期：混合暖色
            let warmth = 1.0 - (twilight - 0.5).abs() * 2.0; // 在中间最暖
            let r = 1.0;
            let g = 0.85 + warmth * 0.15 - warmth * 0.2;
            let b = 0.7 - warmth * 0.3;
            sun_light.color = Color::srgb(r, g.max(0.65), b.max(0.4));
        } else {
            sun_light.color = Color::WHITE;
        }
    }

    if let Ok((mut moon_transform, mut moon_light)) = moon_query.single_mut() {
        moon_transform.translation = Vec3::ZERO;
        moon_transform.rotation = Quat::from_rotation_x(moon_angle);

        let moon_forward_y = moon_transform.forward().y;
        let moon_fade = ((-moon_forward_y + 0.12) / 1.12).clamp(0.0, 1.0);
        moon_light.illuminance = MIN_MOON_ILLUMINANCE + moon_fade * (MAX_MOON_ILLUMINANCE - MIN_MOON_ILLUMINANCE);
    }

    // 相机自动曝光
    // 相机曝光 + 体积雾联动
    let night_factor = time_of_day.night_factor();

    for (mut exposure, fog) in &mut camera_query {
        // 自动曝光
        if sun_fade > 0.0 {
            let computed_ev100 = 11.5 - 5.0 * current_sun_y - 1.5 * current_sun_y * current_sun_y;
            exposure.ev100 = computed_ev100.clamp(5.0, 15.0);
        } else {
            // 夜间降低曝光使场景更暗
            exposure.ev100 = 12.0 + night_factor * 3.0;
        }

        // 体积雾环境光联动
        if let Some(mut vol_fog) = fog {
            let twilight = time_of_day.twilight_factor();
            if twilight > 0.0 && twilight < 1.0 {
                vol_fog.ambient_intensity = TWILIGHT_FOG_AMBIENT;
            } else if night_factor > 0.5 {
                vol_fog.ambient_intensity = NIGHT_FOG_AMBIENT;
            } else {
                vol_fog.ambient_intensity = DAY_FOG_AMBIENT;
            }
        }
    }
}


/// 天体纹理处理系统
pub fn celestial_mesh_system(
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
    mut sun_mesh_query: Query<&mut Transform, (With<SunMesh>, Without<MoonMesh>)>,
    mut moon_mesh_query: Query<&mut Transform, (With<MoonMesh>, Without<SunMesh>)>,
    sun_query: Query<&Transform, (With<Sun>, Without<SunMesh>, Without<MoonMesh>)>,
    moon_query: Query<&Transform, (With<Moon>, Without<SunMesh>, Without<MoonMesh>)>,
) {
    let Ok(camera_transform) = camera_query.single() else { return };
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