use bevy::prelude::*;

use crate::game::simulation::SimulationTransformHistory;
use crate::shared::states::AppState;

/// Marks a visual child whose parent owns an authoritative simulation transform.
#[derive(Component, Debug, Clone, Copy)]
pub struct SimulationPresentation {
    base: Option<Transform>,
    interpolate_rotation: bool,
}

impl SimulationPresentation {
    pub const fn translation_only() -> Self {
        Self {
            base: None,
            interpolate_rotation: false,
        }
    }

    pub const fn full_transform() -> Self {
        Self {
            base: None,
            interpolate_rotation: true,
        }
    }
}

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
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    #[test]
    fn interpolation_plugin_initializes_without_conflicting_transform_queries() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_plugins(ClientInterpolationPlugin);

        app.update();
    }

    #[test]
    fn presentation_offset_keeps_authoritative_transform_unchanged() {
        let authoritative = Transform::from_xyz(2.0, 3.0, 4.0);
        let visual = Transform::from_xyz(1.5, 3.25, 4.0);
        let base = Transform::from_xyz(0.0, 0.75, 0.0);

        let presented = presentation_transform(base, authoritative, visual, false);

        assert_eq!(authoritative.translation, Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(presented.translation, Vec3::new(-0.5, 1.0, 0.0));
    }

    #[test]
    fn full_presentation_interpolates_parent_rotation_before_display_rotation() {
        let authoritative = Transform::from_rotation(Quat::from_rotation_y(1.0));
        let visual = Transform::from_rotation(Quat::from_rotation_y(0.5));
        let base = Transform::from_rotation(Quat::from_rotation_x(0.25));

        let presented = presentation_transform(base, authoritative, visual, true);
        let expected = authoritative.rotation.inverse() * visual.rotation * base.rotation;

        assert!(presented.rotation.dot(expected).abs() > 0.999_99);
    }
}
