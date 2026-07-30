//! 在渲染帧把固定步权威变换插值到独立表现子实体。

use bevy::prelude::*;

use crate::game::simulation::SimulationTransformHistory;
use crate::shared::states::AppState;

/// 标记由父实体持有权威模拟变换的表现子实体。
#[derive(Component, Debug, Clone, Copy)]
pub struct SimulationPresentation {
    base: Option<Transform>,
    interpolate_rotation: bool,
}

impl SimulationPresentation {
    /// 创建只插值位移、保持权威旋转的表现配置。
    pub const fn translation_only() -> Self {
        Self {
            base: None,
            interpolate_rotation: false,
        }
    }

    /// 创建同时插值位移和旋转的表现配置。
    pub const fn full_transform() -> Self {
        Self {
            base: None,
            interpolate_rotation: true,
        }
    }
}

/// 在每个渲染帧把权威变换历史应用到表现子实体。
pub struct ClientInterpolationPlugin;

impl Plugin for ClientInterpolationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            interpolate_simulation_presentations
                .before(bevy::transform::TransformSystems::Propagate)
                .run_if(in_state(AppState::InGame)),
        );
    }
}

fn interpolate_simulation_presentations(
    fixed_time: Res<Time<Fixed>>,
    source_query: Query<(&Transform, &SimulationTransformHistory), Without<SimulationPresentation>>,
    mut presentation_query: Query<
        (&ChildOf, &mut Transform, &mut SimulationPresentation),
        Without<SimulationTransformHistory>,
    >,
) {
    let alpha = fixed_time.overstep_fraction();
    for (parent, mut transform, mut presentation) in &mut presentation_query {
        let Ok((source_transform, history)) = source_query.get(parent.parent()) else {
            continue;
        };
        let base = *presentation.base.get_or_insert(*transform);
        *transform = presentation_transform(
            base,
            *source_transform,
            history.visual_transform(*source_transform, alpha),
            presentation.interpolate_rotation,
        );
    }
}

fn presentation_transform(
    base: Transform,
    authoritative: Transform,
    visual: Transform,
    interpolate_rotation: bool,
) -> Transform {
    let world_delta = visual.translation - authoritative.translation;
    let unrotated_delta = authoritative.rotation.inverse() * world_delta;
    let local_delta = unrotated_delta * reciprocal_scale(authoritative.scale);
    Transform {
        translation: base.translation + local_delta,
        rotation: if interpolate_rotation {
            authoritative.rotation.inverse() * visual.rotation * base.rotation
        } else {
            base.rotation
        },
        scale: base.scale,
    }
}

fn reciprocal_scale(scale: Vec3) -> Vec3 {
    Vec3::new(
        safe_recip(scale.x),
        safe_recip(scale.y),
        safe_recip(scale.z),
    )
}

fn safe_recip(value: f32) -> f32 {
    if value.abs() <= f32::EPSILON {
        0.0
    } else {
        value.recip()
    }
}

#[cfg(test)]
#[path = "../../tests/unit/client/interpolation.rs"]
mod tests;
