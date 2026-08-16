//! 降水粒子表现：读权威天气的降水强度与温度，环绕玩家渲染雨滴或雪花。
//!
//! 纯表现层：只把 `WeatherCell.precipitation` 与 `temperature_c` 转成可见粒子，
//! 不参与任何权威规则。粒子数量随降水强度增减，温度低于冰点时下雪、否则下雨。

use crate::game::player::identity::Player;
use crate::game::world::weather::WeatherCell;
use crate::shared::states::AppState;
use bevy::light::NotShadowCaster;
use bevy::prelude::*;

/// 满强度降水时的最大并发粒子数。
const MAX_DROPS: usize = 240;
/// 粒子环绕玩家的水平半径。
const SPAWN_RADIUS: f32 = 16.0;
/// 粒子相对玩家生成高度范围。
const SPAWN_HEIGHT_MIN: f32 = 12.0;
const SPAWN_HEIGHT_MAX: f32 = 22.0;
/// 雨滴下落速度（米/秒）。
const RAIN_FALL_SPEED: f32 = 20.0;
/// 雪花下落速度（米/秒）。
const SNOW_FALL_SPEED: f32 = 2.2;

/// 降水类型：温度低于冰点下雪，否则下雨。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PrecipitationKind {
    Rain,
    Snow,
}

/// 单颗降水粒子。
#[derive(Component)]
struct PrecipitationParticle {
    kind: PrecipitationKind,
    velocity: Vec3,
    /// 雪花横向摆动相位。
    wobble_phase: f32,
}

/// 降水粒子共享的网格与材质。
#[derive(Resource)]
struct PrecipitationVisuals {
    mesh: Handle<Mesh>,
    rain: Handle<StandardMaterial>,
    snow: Handle<StandardMaterial>,
}

impl FromWorld for PrecipitationVisuals {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(Vec3::ONE));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            mesh,
            rain: materials.add(StandardMaterial {
                base_color: Color::srgb(0.45, 0.65, 0.92),
                perceptual_roughness: 0.55,
                ..default()
            }),
            snow: materials.add(StandardMaterial {
                base_color: Color::srgb(0.95, 0.97, 1.0),
                perceptual_roughness: 0.9,
                ..default()
            }),
        }
    }
}

/// 注册降水粒子发射与更新系统。
pub struct ClientWeatherPlugin;

impl Plugin for ClientWeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PrecipitationVisuals>().add_systems(
            Update,
            (spawn_precipitation_system, update_precipitation_system)
                .chain()
                .run_if(in_state(AppState::InGame)),
        );
    }
}

/// 维持与降水强度匹配的粒子数量；强度上升时增补粒子。
fn spawn_precipitation_system(
    time: Res<Time>,
    weather: Option<Res<WeatherCell>>,
    player: Query<&Transform, With<Player>>,
    visuals: Res<PrecipitationVisuals>,
    existing: Query<(), With<PrecipitationParticle>>,
    mut counter: Local<u64>,
    mut commands: Commands,
) {
    let Some(weather) = weather else {
        return;
    };
    let Ok(player_transform) = player.single() else {
        return;
    };
    if weather.precipitation <= 0.01 {
        return;
    }

    let target = (weather.precipitation * MAX_DROPS as f32) as usize;
    let current = existing.iter().count();
    if current >= target {
        return;
    }

    let kind = if weather.temperature_c < 0.0 {
        PrecipitationKind::Snow
    } else {
        PrecipitationKind::Rain
    };
    let elapsed = time.elapsed_secs() as u64;

    for _ in current..target {
        *counter = counter.wrapping_add(1);
        let angle = noise01(*counter, elapsed) * std::f32::consts::TAU;
        let radius = SPAWN_RADIUS * noise01(*counter, elapsed + 1).sqrt();
        let x = player_transform.translation.x + angle.cos() * radius;
        let z = player_transform.translation.z + angle.sin() * radius;
        let y = player_transform.translation.y
            + SPAWN_HEIGHT_MIN
            + (SPAWN_HEIGHT_MAX - SPAWN_HEIGHT_MIN) * noise01(*counter, elapsed + 2);

        let (scale, velocity, material) = match kind {
            PrecipitationKind::Rain => (
                Vec3::new(0.03, 0.2, 0.03),
                Vec3::new(0.0, -RAIN_FALL_SPEED, 0.0),
                visuals.rain.clone(),
            ),
            PrecipitationKind::Snow => (
                Vec3::splat(0.07),
                Vec3::new(0.0, -SNOW_FALL_SPEED, 0.0),
                visuals.snow.clone(),
            ),
        };

        commands.spawn((
            Name::new("PrecipitationParticle"),
            PrecipitationParticle {
                kind,
                velocity,
                wobble_phase: noise01(*counter, elapsed + 3) * std::f32::consts::TAU,
            },
            Mesh3d(visuals.mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(Vec3::new(x, y, z)).with_scale(scale),
            Visibility::Inherited,
            NotShadowCaster,
        ));
    }
}

/// 驱动雨滴下落与雪花飘落，落到玩家脚下后回收。
fn update_precipitation_system(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut query: Query<(Entity, &mut Transform, &mut PrecipitationParticle), Without<Player>>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player.single() else {
        return;
    };
    let dt = time.delta_secs();
    let floor_y = player_transform.translation.y - 2.0;

    for (entity, mut transform, mut particle) in &mut query {
        if particle.kind == PrecipitationKind::Snow {
            particle.wobble_phase += dt * 2.0;
            let sway = particle.wobble_phase.sin() * 0.9;
            transform.translation.x += sway * dt;
            transform.translation.z += (particle.wobble_phase * 1.3).cos() * 0.7 * dt;
        }
        transform.translation += particle.velocity * dt;

        if transform.translation.y < floor_y {
            commands.entity(entity).despawn();
        }
    }
}

/// 由计数器与时间产生 0~1 的稳定伪随机数（表现层无需确定性，仅求分布均匀）。
fn noise01(seed: u64, stream: u64) -> f32 {
    let mut value = seed
        .wrapping_add(stream.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((value ^ (value >> 31)) as u32) as f32 / u32::MAX as f32
}

#[cfg(test)]
#[path = "../../../tests/unit/client/weather/mod.rs"]
mod tests;
