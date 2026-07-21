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
