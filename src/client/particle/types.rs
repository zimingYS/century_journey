//! 粒子反馈的数据类型与视觉资源。

use bevy::prelude::*;

/// 粒子材质类别，决定反馈粒子的颜色与材质句柄。
#[derive(Debug, Clone, Copy)]
pub(super) enum ParticleKind {
    Stone,
    Earth,
    Wood,
    Spark,
    Hit,
}

/// 粒子系统共享的网格与材质句柄。
#[derive(Resource)]
pub(super) struct ParticleVisuals {
    pub(super) mesh: Handle<Mesh>,
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
    /// 返回指定粒子类别对应的材质句柄。
    pub(super) fn material(&self, kind: ParticleKind) -> Handle<StandardMaterial> {
        match kind {
            ParticleKind::Stone => self.stone.clone(),
            ParticleKind::Earth => self.earth.clone(),
            ParticleKind::Wood => self.wood.clone(),
            ParticleKind::Spark => self.spark.clone(),
            ParticleKind::Hit => self.hit.clone(),
        }
    }
}

/// 单个反馈粒子的运动与寿命状态。
#[derive(Component)]
pub(super) struct FeedbackParticle {
    pub(super) velocity: Vec3,
    pub(super) age: f32,
    pub(super) lifetime: f32,
    pub(super) initial_scale: f32,
    pub(super) spin: f32,
}
