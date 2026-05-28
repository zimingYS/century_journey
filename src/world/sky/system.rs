use bevy::camera::Exposure;
use crate::world::sky::component::*;
use crate::world::time::TimeOfDay;
use bevy::light::{CascadeShadowConfigBuilder, VolumetricLight};
use bevy::prelude::*;

pub fn setup_sky_system(mut commands: Commands) {
    // 构造级联阴影
    let cascade_shadow_config = CascadeShadowConfigBuilder {
        // 第一个阴影级联的远边界
        first_cascade_far_bound: 16.0,
        // 阴影的最大渲染距离
        maximum_distance: 64.0,
        ..default()
    }
        .build();

    // 生成太阳光
    commands.spawn((
        Sun,
        DirectionalLight{
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
        // 使用体积光计算光线
        VolumetricLight,
        cascade_shadow_config.clone(),
    ));

    // 生成月亮
    commands.spawn((
        Moon,
        DirectionalLight {
            color: Color::srgb(0.8, 0.85, 1.0),
            illuminance: light_consts::lux::FULL_MOON_NIGHT,
            shadows_enabled: true,
            ..default()
        },
        Transform::IDENTITY,
        cascade_shadow_config,
    ));
}

pub fn atmosphere_system(
    time_of_day: Res<TimeOfDay>,
    mut sun_query: Query<(&mut Transform, &mut DirectionalLight), (With<Sun>, Without<Moon>)>,
    mut moon_query: Query<(&mut Transform, &mut DirectionalLight), (With<Moon>, Without<Sun>)>,
    mut camera_query: Query<&mut Exposure, With<Camera3d>>,
){
    // 太阳当前弧度角 (0.0 到 2π)
    let sun_angle = ((time_of_day.current_time + 6.0) / 24.0) * std::f32::consts::TAU;
    // 月亮与太阳永远保持 180 度正对立
    let moon_angle = sun_angle + std::f32::consts::PI;

    let mut current_sun_y = 0.0;
    let mut sun_fade = 0.0;

    if let Ok((mut sun_transform, mut sun_light)) = sun_query.single_mut() {
        sun_transform.translation = Vec3::ZERO;
        // 绕X轴旋转太阳
        sun_transform.rotation = Quat::from_rotation_x(sun_angle);

        // 获取太阳高度
        let sun_forward_y = sun_transform.forward().y;
        current_sun_y = sun_forward_y;

        sun_fade = ((-sun_forward_y + 0.12) / 1.12).clamp(0.0, 1.0);

        sun_light.illuminance = sun_fade * light_consts::lux::RAW_SUNLIGHT;
    }

    if let Ok((mut moon_transform, _moon_light)) = moon_query.single_mut() {
        moon_transform.translation = Vec3::ZERO;
        moon_transform.rotation = Quat::from_rotation_x(moon_angle);
    }

    // 相机自动曝光
    for mut exposure in &mut camera_query {
        if sun_fade > 0.0 {
            let computed_ev100 = 11.5 - 5.0 * current_sun_y - 1.5 * current_sun_y * current_sun_y;
            exposure.ev100 = computed_ev100.clamp(5.0, 15.0);
        }
    }
}