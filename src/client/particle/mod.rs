//! 轻量级客户端粒子反馈。
//!
//! 粒子由已经发生的方块事件和动画标记生成，不承担命中或方块变更判定。

use bevy::light::NotShadowCaster;
use bevy::prelude::*;

use crate::content::block::event::{BlockBreakEvent, BlockPlaceEvent};
use crate::content::block::registry::BlockRegistry;
use crate::content::block::sound::SoundMaterial;
use crate::game::player::components::Player;
use crate::game::player::events::AttackEvent;
use crate::game::player::model::animation::{AnimationMarkerEvent, AnimationMarkerKind};
use crate::game::player::systems::raycast::TargetVoxel;
use crate::game::world::block_ops::get_voxel_at_world;
use crate::game::world::storage::WorldStorage;
use crate::shared::states::AppState;

#[derive(Debug, Clone, Copy)]
enum ParticleKind {
    Stone,
    Earth,
    Wood,
    Spark,
    Hit,
}

#[derive(Resource)]
struct ParticleVisuals {
    mesh: Handle<Mesh>,
    stone: Handle<StandardMaterial>,
    earth: Handle<StandardMaterial>,
    wood: Handle<StandardMaterial>,
    spark: Handle<StandardMaterial>,
    hit: Handle<StandardMaterial>,
}

impl FromWorld for ParticleVisuals {
    fn from_world(world: &mut World) -> Self {
        let mesh = world
            .resource_mut::<Assets<Mesh>>()
            .add(Cuboid::from_size(Vec3::ONE));
        let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
        Self {
            mesh,
            stone: materials.add(particle_material(Color::srgb(0.48, 0.52, 0.56))),
            earth: materials.add(particle_material(Color::srgb(0.40, 0.27, 0.14))),
            wood: materials.add(particle_material(Color::srgb(0.52, 0.32, 0.13))),
            spark: materials.add(particle_material(Color::srgb(0.96, 0.71, 0.18))),
            hit: materials.add(particle_material(Color::srgb(0.72, 0.12, 0.09))),
        }
    }
}

fn particle_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.92,
        ..default()
    }
}

impl ParticleVisuals {
    fn material(&self, kind: ParticleKind) -> Handle<StandardMaterial> {
        match kind {
            ParticleKind::Stone => self.stone.clone(),
            ParticleKind::Earth => self.earth.clone(),
            ParticleKind::Wood => self.wood.clone(),
            ParticleKind::Spark => self.spark.clone(),
            ParticleKind::Hit => self.hit.clone(),
        }
    }
}

#[derive(Component)]
struct FeedbackParticle {
    velocity: Vec3,
    age: f32,
    lifetime: f32,
    initial_scale: f32,
    spin: f32,
}

const PARTICLE_GRAVITY: f32 = 7.8;
const PARTICLE_DRAG: f32 = 1.7;
const MAX_PARTICLE_STEP_SECONDS: f32 = 1.0 / 120.0;

pub struct ClientParticlePlugin;

impl Plugin for ClientParticlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ParticleVisuals>().add_systems(
            Update,
            (
                spawn_block_particles_system,
                spawn_action_particles_system,
                update_feedback_particles_system,
            )
                .run_if(in_state(AppState::InGame)),
        );
    }
}

fn spawn_block_particles_system(
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

fn spawn_action_particles_system(
    mut reader: MessageReader<AnimationMarkerEvent>,
    mut attack_reader: MessageReader<AttackEvent>,
    target: Res<TargetVoxel>,
    world: Res<WorldStorage>,
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
                let block_id = get_voxel_at_world(hit.hit_pos, &world);
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

fn update_feedback_particles_system(
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

fn advance_particle_motion(
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

#[cfg(test)]
mod tests {
    use super::*;

    fn simulate_particle(fps: u32) -> (Vec3, Vec3) {
        let mut particle = FeedbackParticle {
            velocity: Vec3::new(2.0, 3.0, -1.0),
            age: 0.0,
            lifetime: 2.0,
            initial_scale: 0.1,
            spin: 0.0,
        };
        let mut transform = Transform::default();
        let delta = 1.0 / fps as f32;
        for _ in 0..fps {
            advance_particle_motion(&mut particle, &mut transform, delta);
        }
        (transform.translation, particle.velocity)
    }

    #[test]
    fn particle_motion_is_stable_across_render_rates() {
        let at_10 = simulate_particle(10);
        let at_20 = simulate_particle(20);
        let at_60 = simulate_particle(60);
        let at_144 = simulate_particle(144);

        assert!(at_10.0.distance(at_20.0) < 0.01);
        assert!(at_20.0.distance(at_60.0) < 0.01);
        assert!(at_60.0.distance(at_144.0) < 0.01);
        assert!(at_10.1.distance(at_20.1) < 0.01);
        assert!(at_20.1.distance(at_60.1) < 0.01);
        assert!(at_60.1.distance(at_144.1) < 0.01);
    }
}
