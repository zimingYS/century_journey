//! 降水粒子表现的数据类型与视觉资源。

use bevy::prelude::*;

/// 降水类型：温度低于冰点下雪，否则下雨。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrecipitationKind {
    Rain,
    Snow,
}

/// 单颗降水粒子。
#[derive(Component)]
pub(super) struct PrecipitationParticle {
    pub(super) kind: PrecipitationKind,
    pub(super) velocity: Vec3,
    /// 雪花横向摆动相位。
    pub(super) wobble_phase: f32,
}

/// 降水粒子共享的网格与材质。
#[derive(Resource)]
pub(super) struct PrecipitationVisuals {
    pub(super) mesh: Handle<Mesh>,
    pub(super) rain: Handle<StandardMaterial>,
    pub(super) snow: Handle<StandardMaterial>,
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
