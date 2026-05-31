use crate::player::components::Player;
use crate::core::input_block::InputBlocked;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(Component)]
pub struct FpsCamera{
    pub mouse_sensitivity: f32,
    pub pitch: f32,
}

impl Default for FpsCamera{
    fn default() -> FpsCamera{
        Self{
            mouse_sensitivity: 0.015,
            pitch: 0.0,
        }
    }
}

pub fn player_look_system(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut camera_query: Query<(&mut Transform,&mut FpsCamera),Without<Player>>,
    input_blocked: Res<InputBlocked>,
){
    if input_blocked.0 { return; }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read(){
        delta += event.delta;
    }

    if delta == Vec2::ZERO {
        return;
    }

    // 左右旋转
    if let Ok(mut player_transform) = player_query.single_mut() {
        player_transform.rotate_y(-delta.x * 0.0015);
    }

    // 上下旋转
    if let Ok((mut camera_transform,mut fps_camera)) = camera_query.single_mut() {
        camera_transform.rotate_local_x(-delta.y * 0.0015);
    }
}

pub fn convert_mouse_lock_on_startup(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
){
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else { return };

    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;
}

pub fn toggle_mouse_lock_system(
    mut cursor_options_query: Query<&mut CursorOptions, With<PrimaryWindow>>,
    keyboard_input: Res<ButtonInput<KeyCode>>
){
    let Ok(mut cursor_options) = cursor_options_query.single_mut() else { return };

    if keyboard_input.just_pressed(KeyCode::Escape){
        if cursor_options.grab_mode == CursorGrabMode::Locked {
            cursor_options.visible = true;
            cursor_options.grab_mode = CursorGrabMode::None;
        }else {
            cursor_options.visible = false;
            cursor_options.grab_mode = CursorGrabMode::Locked;
        }
    }
}