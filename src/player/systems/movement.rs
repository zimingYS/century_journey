use crate::player::components::{Player, PlayerMovement};
use bevy::prelude::*;

pub fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut movement_query: Query<(&mut Transform, &PlayerMovement),With<Player>>,
){
    let Ok((mut transform, player_movement)) = movement_query
        .single_mut()
    else { return };

    let mut direction = Vec3::ZERO;

    let forward = transform.forward();
    let right = transform.right();

    if keyboard_input.pressed(KeyCode::KeyW) { direction += forward.as_vec3(); }
    if keyboard_input.pressed(KeyCode::KeyS) { direction -= forward.as_vec3(); }
    if keyboard_input.pressed(KeyCode::KeyD) { direction += right.as_vec3(); }
    if keyboard_input.pressed(KeyCode::KeyA) { direction -= right.as_vec3(); }
    if keyboard_input.pressed(KeyCode::Space) { direction += Vec3::Y; }
    if keyboard_input.pressed(KeyCode::ControlLeft) { direction -= Vec3::Y; }

    if direction != Vec3::ZERO {
        // direction.y = 0.0;
        direction = direction.normalize();

        let current_speed = if keyboard_input.pressed(KeyCode::ShiftLeft){
            player_movement.movement_speed * player_movement.sprint_factor
        }else {
            player_movement.movement_speed
        };

        transform.translation += direction * current_speed * time.delta_secs();
    }
}