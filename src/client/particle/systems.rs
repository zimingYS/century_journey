//! 粒子发射、运动推进与回收系统的实现。

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::client::particle::types::{FeedbackParticle, ParticleKind, ParticleVisuals};
use crate::client::player::model::animation::{AnimationMarkerEvent, AnimationMarkerKind};
use crate::content::block::event::{BlockBreakEvent, BlockPlaceEvent};
use crate::content::block::registry::BlockRegistry;
use crate::content::block::sound::SoundMaterial;
use crate::game::player::combat::events::AttackEvent;
use crate::game::player::identity::Player;
use crate::game::player::interaction::targeting::TargetVoxel;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::state::WorldState;

const PARTICLE_GRAVITY: f32 = 7.8;
const PARTICLE_DRAG: f32 = 1.7;
const MAX_PARTICLE_STEP_SECONDS: f32 = 1.0 / 120.0;

/// 消费方块破坏与放置事件，发射对应材质的粒子。
pub(super) fn spawn_block_particles_system(
    mut break_reader: MessageReader<BlockBreakEvent>,
    mut place_reader: MessageReader<BlockPlaceEvent>,
    registry: Option<Res<BlockRegistry>>,
    visuals: Res<ParticleVisuals>,
    mut commands: Commands,
) {
    for event in break_reader.read() {
        let kind = registry
            .as_deref()
            .and_then(|registry| registry.get(event.block_id))
            .map(|block| particle_kind(block.sound.sound_material))
            .unwrap_or(ParticleKind::Stone);
        spawn_burst(
            &mut commands,
            &visuals,
            event.world_pos.as_vec3() + Vec3::splat(0.5),
            kind,
            14,
            position_seed(event.world_pos, event.block_id),
            2.7,
        );
    }

    for event in place_reader.read() {
        let kind = registry
            .as_deref()
            .and_then(|registry| registry.get(event.block_id))
            .map(|block| particle_kind(block.sound.sound_material))
            .unwrap_or(ParticleKind::Stone);
        let surface =
            event.world_pos.as_vec3() + Vec3::splat(0.5) - event.face_normal.as_vec3() * 0.48;
        spawn_burst(
            &mut commands,
            &visuals,
            surface,
            kind,
            7,
            position_seed(event.world_pos, event.block_id).wrapping_add(91),
            1.35,
        );
    }
}

/// 消费攻击与动画标记事件，发射命中与采矿粒子。
///
/// 动画、攻击和世界读模型共同决定一次粒子反馈，参数仅在表现帧内读取。
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_action_particles_system(
    mut reader: MessageReader<AnimationMarkerEvent>,
    mut attack_reader: MessageReader<AttackEvent>,
    target: Res<TargetVoxel>,
    world_state: Res<WorldState>,
    registry: Option<Res<BlockRegistry>>,
    player_query: Query<&GlobalTransform, With<Player>>,
    visuals: Res<ParticleVisuals>,
    mut commands: Commands,
) {
    // 红色命中粒子只由已经找到目标的攻击事件驱动，空挥不产生“血液”反馈。
    for attack in attack_reader.read() {
        if attack.attacker == attack.target || attack.amount <= 0.0 {
            continue;
        }
        let Ok(transform) = player_query.get(attack.target) else {
            continue;
        };
        spawn_burst(
            &mut commands,
            &visuals,
            transform.translation() + Vec3::Y * 0.9,
            ParticleKind::Hit,
            6,
            attack.attacker.to_bits() ^ attack.target.to_bits(),
            1.7,
        );
    }

    for event in reader.read() {
        match event.marker {
            AnimationMarkerKind::MiningSwing => {
                let Some(hit) = target.result.as_ref() else {
                    continue;
                };
                let block_id = get_voxel_at_world(hit.hit_pos, &world_state);
                let kind = registry
                    .as_deref()
                    .and_then(|registry| registry.get(block_id))
                    .map(|block| particle_kind(block.sound.sound_material))
                    .unwrap_or(ParticleKind::Stone);
                let origin = hit.hit_pos.as_vec3() + Vec3::splat(0.5) + hit.normal.as_vec3() * 0.51;
                spawn_burst(
                    &mut commands,
                    &visuals,
                    origin,
                    kind,
                    4,
                    position_seed(hit.hit_pos, event.cycle as u16),
                    1.15,
                );
            }
            AnimationMarkerKind::AttackHit
            | AnimationMarkerKind::PlaceCommit
            | AnimationMarkerKind::UseCommit => {}
        }
    }
}

/// 推进粒子运动、缩放与寿命，到期回收。
pub(super) fn update_feedback_particles_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut FeedbackParticle)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut particle) in &mut query {
        particle.age += delta;
        if particle.age >= particle.lifetime {
            commands.entity(entity).despawn();
            continue;
        }

        advance_particle_motion(&mut particle, &mut transform, delta);
        transform.rotate_y(particle.spin * delta);
        transform.rotate_x(particle.spin * 0.63 * delta);

        let remaining = 1.0 - particle.age / particle.lifetime;
        transform.scale = Vec3::splat(particle.initial_scale * remaining.max(0.08));
    }
}

/// 推进粒子的重力与阻力运动；固定子步保证不同渲染帧率下轨迹一致。
pub(super) fn advance_particle_motion(
    particle: &mut FeedbackParticle,
    transform: &mut Transform,
    delta_seconds: f32,
) {
    if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
        return;
    }
    let mut remaining = delta_seconds;
    while remaining > f32::EPSILON {
        let step = remaining.min(MAX_PARTICLE_STEP_SECONDS);
        particle.velocity.y -= PARTICLE_GRAVITY * step;
        particle.velocity *= (-PARTICLE_DRAG * step).exp();
        transform.translation += particle.velocity * step;
        remaining -= step;
    }
}

fn spawn_burst(
    commands: &mut Commands,
    visuals: &ParticleVisuals,
    origin: Vec3,
    kind: ParticleKind,
    count: usize,
    seed: u64,
    speed: f32,
) {
    for index in 0..count {
        let x = signed_noise(seed, index as u64 * 3);
        let y = noise01(seed, index as u64 * 3 + 1);
        let z = signed_noise(seed, index as u64 * 3 + 2);
        let direction = Vec3::new(x, 0.35 + y, z).normalize_or_zero();
        let initial_scale = 0.055 + noise01(seed ^ 0x91E1, index as u64) * 0.075;
        let lifetime = 0.32 + noise01(seed ^ 0xA4B7, index as u64) * 0.46;
        let spawn_offset = Vec3::new(x, signed_noise(seed ^ 0x38, index as u64), z) * 0.18;

        commands.spawn((
            Name::new("FeedbackParticle"),
            FeedbackParticle {
                velocity: direction * speed * (0.72 + y * 0.55),
                age: 0.0,
                lifetime,
                initial_scale,
                spin: signed_noise(seed ^ 0xD2, index as u64) * 8.0,
            },
            Mesh3d(visuals.mesh.clone()),
            MeshMaterial3d(visuals.material(kind)),
            Transform::from_translation(origin + spawn_offset)
                .with_scale(Vec3::splat(initial_scale)),
            Visibility::Inherited,
            NotShadowCaster,
        ));
    }
}

fn particle_kind(material: SoundMaterial) -> ParticleKind {
    match material {
        SoundMaterial::Dirt
        | SoundMaterial::Grass
        | SoundMaterial::Sand
        | SoundMaterial::Cloth
        | SoundMaterial::Snow
        | SoundMaterial::Water => ParticleKind::Earth,
        SoundMaterial::Wood => ParticleKind::Wood,
        SoundMaterial::Metal | SoundMaterial::Glass => ParticleKind::Spark,
        SoundMaterial::Stone => ParticleKind::Stone,
    }
}

fn position_seed(position: IVec3, salt: u16) -> u64 {
    (position.x as u64).wrapping_mul(73_856_093)
        ^ (position.y as u64).wrapping_mul(19_349_663)
        ^ (position.z as u64).wrapping_mul(83_492_791)
        ^ salt as u64
}

fn noise01(seed: u64, stream: u64) -> f32 {
    let mut value = seed
        .wrapping_add(stream.wrapping_mul(0x9E37_79B9_7F4A_7C15))
        .wrapping_add(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    ((value ^ (value >> 31)) as u32) as f32 / u32::MAX as f32
}

fn signed_noise(seed: u64, stream: u64) -> f32 {
    noise01(seed, stream) * 2.0 - 1.0
}
