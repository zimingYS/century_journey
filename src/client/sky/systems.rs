//! 根据连续视觉时间更新天空颜色、日月轨迹和主方向光。

use super::constants::*;
use crate::app::flow::GameSettings;
use crate::client::presentation::TimeOfDay;
use crate::client::sky::components::*;
use crate::client::sky::texture;
use crate::game::player::identity::LocalPlayer;
use crate::game::world::lighting::WorldLighting;
use crate::shared::voxel::CHUNK_SIZE;
use bevy::camera::{Exposure, visibility::RenderLayers};
use bevy::ecs::system::SystemParam;
use bevy::light::atmosphere::ScatteringMedium;
use bevy::light::{
    Atmosphere, AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, GlobalAmbientLight,
    NotShadowCaster, NotShadowReceiver, VolumetricFog, VolumetricLight,
};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use rand::{RngExt, SeedableRng};
use std::f32::consts::TAU;

/// 玩家眼睛相对权威位置的高度，用于采样当前空间的天空可见度。
const PLAYER_EYE_HEIGHT: f32 = 0.8;
/// 天空可见度每秒趋近目标的速率，消除跨体素和光照提交造成的整屏跳变。
const CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND: f32 = 5.0;
/// 曝光适应比日月直射光更慢，避免洞口附近少量天空光改变整屏亮度。
const EXPOSURE_VISIBILITY_RESPONSE_PER_SECOND: f32 = 1.8;
/// 单帧最多消费的真实时间，避免切出窗口后恢复时直接跨完整个过渡。
const MAX_VISIBILITY_STEP_SECONDS: f32 = 0.1;
/// 曝光判断在玩家周围取样的水平半径，单格天井不会被误判成露天。
const EXPOSURE_SAMPLE_RADIUS: f32 = 1.5;
/// 进入露天曝光所需的周围天空可见度。
const EXPOSURE_OPEN_THRESHOLD: f32 = 0.68;
/// 回到洞穴曝光的天空可见度，和进入阈值构成迟滞区间。
const EXPOSURE_CAVE_THRESHOLD: f32 = 0.32;
/// 方向光阴影达到较高露天可见度后才启用，避免洞口边缘频繁创建阴影资源。
const SHADOW_MAP_ENABLE_THRESHOLD: f32 = 0.42;
/// 阴影已启用时允许的最低露天可见度；与启用阈值分离以形成迟滞区间。
const SHADOW_MAP_DISABLE_THRESHOLD: f32 = 0.24;

/// 客户端日月直射光的连续可见度状态。
///
/// 目标值来自权威天空光场，但只影响渲染帧表现；光照数据暂缺时保持已有目标，
/// 不能把“尚未加载”误判为露天。
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CelestialVisibilityState {
    current: f32,
    target: f32,
    exposure_current: f32,
    exposure_target: f32,
    shadow_maps_current: bool,
}

impl Default for CelestialVisibilityState {
    fn default() -> Self {
        Self {
            current: 1.0,
            target: 1.0,
            exposure_current: 1.0,
            exposure_target: 1.0,
            shadow_maps_current: false,
        }
    }
}

impl CelestialVisibilityState {
    fn update(
        &mut self,
        sample: Option<f32>,
        exposure_sample: Option<f32>,
        delta_seconds: f32,
    ) -> (f32, f32) {
        if let Some(sample) = sample {
            self.target = sample.clamp(0.0, 1.0);
        }
        if let Some(sample) = exposure_sample {
            self.exposure_target = exposure_visibility_target(sample, self.exposure_target);
        }
        self.current = visibility_step(
            self.current,
            self.target,
            delta_seconds,
            CELESTIAL_VISIBILITY_RESPONSE_PER_SECOND,
        );
        self.exposure_current = visibility_step(
            self.exposure_current,
            self.exposure_target,
            delta_seconds,
            EXPOSURE_VISIBILITY_RESPONSE_PER_SECOND,
        );
        (self.current, self.exposure_current)
    }

    /// 按稳定的天空采样切换方向光阴影资格；迟滞避免洞口边缘反复重建级联资源。
    fn update_shadow_maps(&mut self, visibility: f32, samples_ready: bool) -> bool {
        let visibility = visibility.clamp(0.0, 1.0);
        self.shadow_maps_current = if !samples_ready {
            false
        } else if self.shadow_maps_current {
            visibility >= SHADOW_MAP_DISABLE_THRESHOLD
        } else {
            visibility >= SHADOW_MAP_ENABLE_THRESHOLD
        };
        self.shadow_maps_current
    }
}

/// 聚合天空表现系统每帧只读的时间、设置和权威光场输入。
#[derive(SystemParam)]
pub(crate) struct AtmosphereInputs<'w> {
    real_time: Res<'w, Time<Real>>,
    time_of_day: Res<'w, TimeOfDay>,
    settings: Res<'w, GameSettings>,
    lighting: Option<Res<'w, WorldLighting>>,
}

/// 创建天空盒、太阳、月亮和主方向光实体。
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
            // 天空光快照尚未就绪时不把未知空间当作露天，首帧由门控系统决定是否开启。
            shadow_maps_enabled: false,
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
            shadow_maps_enabled: false,
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

/// 根据连续视觉时间更新天空颜色、日月位置和环境亮度。
/// 日月、相机曝光和雾效必须使用同一视觉时间快照，查询过滤器保持显式。
#[allow(clippy::type_complexity)]
pub(crate) fn atmosphere_system(
    inputs: AtmosphereInputs,
    mut visibility_state: ResMut<CelestialVisibilityState>,
    mut ambient_light: ResMut<GlobalAmbientLight>,
    mut sun_query: Query<
        (&mut Transform, &mut DirectionalLight),
        (With<Sun>, Without<Moon>, Without<LocalPlayer>),
    >,
    mut moon_query: Query<
        (&mut Transform, &mut DirectionalLight),
        (With<Moon>, Without<Sun>, Without<LocalPlayer>),
    >,
    player_query: Query<&Transform, (With<LocalPlayer>, Without<Sun>, Without<Moon>)>,
    mut camera_query: Query<
        (
            &mut Exposure,
            Option<&mut VolumetricFog>,
            Option<&mut DistanceFog>,
            Option<&mut AtmosphereEnvironmentMapLight>,
        ),
        With<crate::client::camera::FpsCamera>,
    >,
) {
    let AtmosphereInputs {
        real_time,
        time_of_day,
        settings,
        lighting,
    } = inputs;
    // 太阳当前弧度角 (0.0 到 2π)
    let sun_angle = ((time_of_day.current_time + 6.0) / 24.0) * TAU;
    // 月亮与太阳永远保持 180 度正对立
    let moon_angle = sun_angle + std::f32::consts::PI;
    let player_eye = player_query
        .single()
        .ok()
        .map(|transform| transform.translation + Vec3::Y * PLAYER_EYE_HEIGHT);
    let visibility_sample =
        player_eye.and_then(|position| celestial_visibility_at(lighting.as_deref(), position));
    let exposure_sample =
        player_eye.and_then(|position| exposure_sky_openness_at(lighting.as_deref(), position));
    let (_celestial_visibility, exposure_visibility) =
        visibility_state.update(visibility_sample, exposure_sample, real_time.delta_secs());
    // 亮度与相机曝光采用抗局部亮点的 exposure 可见度；阴影还要求眼前实际存在
    // 天空光，避免暗处曝光抬升后仍在洞穴中分配并投射方向光阴影图。
    let directional_shadow_visibility = exposure_sample
        .unwrap_or(0.0)
        .min(visibility_sample.unwrap_or(0.0));
    let shadow_maps_visible = visibility_state.update_shadow_maps(
        directional_shadow_visibility,
        visibility_sample.is_some() && exposure_sample.is_some(),
    );

    let mut current_sun_y = 0.0;
    let mut sun_fade = 0.0;

    if let Ok((mut sun_transform, mut sun_light)) = sun_query.single_mut() {
        sun_transform.translation = Vec3::ZERO;
        sun_transform.rotation = Quat::from_rotation_x(sun_angle);

        let sun_forward_y = sun_transform.forward().y;
        current_sun_y = sun_forward_y;

        // 太阳高度淡出
        sun_fade = ((-sun_forward_y + 0.12) / 1.12).clamp(0.0, 1.0);
        sun_light.illuminance = sun_fade * exposure_visibility * DAY_SUN_ILLUMINANCE;
        // 阴影布尔开关使用经过空间中值、迟滞和跨帧平滑后的门控，不随单格采样抖动。
        sun_light.shadow_maps_enabled = sun_fade > 0.02 && shadow_maps_visible;

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
        moon_light.illuminance = exposure_visibility
            * (MIN_MOON_ILLUMINANCE + moon_fade * (MAX_MOON_ILLUMINANCE - MIN_MOON_ILLUMINANCE));
        moon_light.shadow_maps_enabled = moon_fade > 0.02 && sun_fade < 0.18 && shadow_maps_visible;
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
        // 封闭洞穴必须采用暗处曝光，否则白天的露天 EV 会把火把压到近乎不可见。
        // 输入已经过空间插值和跨帧平滑，因此进出洞口只会连续适应，不再跨格闪烁。
        exposure.ev100 =
            visibility_exposure_ev100(sun_fade * exposure_visibility, current_sun_y, night_factor);

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

fn celestial_visibility_at(lighting: Option<&WorldLighting>, position: Vec3) -> Option<f32> {
    let lighting = lighting?;
    let center_voxel = position.floor().as_ivec3();
    let center_visibility = lighting
        .light_cell_at_world(center_voxel)
        .map(sky_visibility)?;

    // 光值定义在体素中心；对相邻八个中心做三线性插值，跨方块边界时连续变化。
    let grid_position = position - Vec3::splat(0.5);
    let base = grid_position.floor().as_ivec3();
    let fraction = (grid_position - base.as_vec3()).clamp(Vec3::ZERO, Vec3::ONE);
    let mut visibility = 0.0;
    for y in 0..=1 {
        for z in 0..=1 {
            for x in 0..=1 {
                let offset = IVec3::new(x, y, z);
                let weight = if x == 0 { 1.0 - fraction.x } else { fraction.x }
                    * if y == 0 { 1.0 - fraction.y } else { fraction.y }
                    * if z == 0 { 1.0 - fraction.z } else { fraction.z };
                let sample = lighting
                    .light_cell_at_world(base + offset)
                    .map(sky_visibility)
                    .unwrap_or(center_visibility);
                visibility += sample * weight;
            }
        }
    }
    Some(visibility.clamp(0.0, 1.0))
}

fn sky_visibility(cell: crate::game::world::lighting::chunk_light::LightCell) -> f32 {
    f32::from(cell.sky.r.max(cell.sky.g).max(cell.sky.b)) / 15.0
}

/// 取中心和四个水平邻点的中位数；只有多数取样可见天空才改变整屏曝光。
fn exposure_sky_openness_at(lighting: Option<&WorldLighting>, position: Vec3) -> Option<f32> {
    let mut samples = [
        celestial_visibility_at(lighting, position)?,
        celestial_visibility_at(lighting, position + Vec3::X * EXPOSURE_SAMPLE_RADIUS)?,
        celestial_visibility_at(lighting, position - Vec3::X * EXPOSURE_SAMPLE_RADIUS)?,
        celestial_visibility_at(lighting, position + Vec3::Z * EXPOSURE_SAMPLE_RADIUS)?,
        celestial_visibility_at(lighting, position - Vec3::Z * EXPOSURE_SAMPLE_RADIUS)?,
    ];
    samples.sort_by(f32::total_cmp);
    Some(samples[2])
}

fn exposure_visibility_target(sample: f32, previous_target: f32) -> f32 {
    if sample >= EXPOSURE_OPEN_THRESHOLD {
        sample.clamp(0.0, 1.0)
    } else if sample <= EXPOSURE_CAVE_THRESHOLD {
        0.0
    } else {
        previous_target
    }
}

fn visibility_step(current: f32, target: f32, delta_seconds: f32, response: f32) -> f32 {
    let delta_seconds = delta_seconds.clamp(0.0, MAX_VISIBILITY_STEP_SECONDS);
    let blend = 1.0 - (-response * delta_seconds).exp();
    (current + (target - current) * blend).clamp(0.0, 1.0)
}

/// 离开世界时清除局部天空可见度，避免下一存档继承洞穴或露天状态。
pub(super) fn reset_celestial_visibility_system(
    mut visibility_state: ResMut<CelestialVisibilityState>,
) {
    *visibility_state = CelestialVisibilityState::default();
}

fn visibility_exposure_ev100(visible_sun_fade: f32, sun_y: f32, night_factor: f32) -> f32 {
    let sun_height = (-sun_y).clamp(0.0, 1.0);
    let daylight_ev100 = 11.8 + 2.3 * sun_height;
    let twilight_ev100 = NIGHT_EXPOSURE_EV100 + (1.0 - night_factor.clamp(0.0, 1.0)) * 1.8;
    let daylight_mix = smoothstep((visible_sun_fade / 0.25).clamp(0.0, 1.0));
    twilight_ev100 + (daylight_ev100 - twilight_ev100) * daylight_mix
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

/// 天体纹理处理系统
/// 日月标记通过互斥过滤器保证各自只跟随对应光源。
#[allow(clippy::type_complexity)]
pub fn celestial_mesh_system(
    camera_query: Query<&GlobalTransform, With<crate::client::camera::FpsCamera>>,
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
