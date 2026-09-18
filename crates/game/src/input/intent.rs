use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct PlayerIntent {
    /// 移动量
    pub(crate) move_axis: Vec3,
    /// 是否移动加速
    pub(crate) sprint: bool,
    /// 本帧鼠标位移
    pub(crate) look_delta: Vec2,
}

impl Default for PlayerIntent {
    fn default() -> Self {
        Self {
            move_axis: Vec3::ZERO,
            sprint: false,
            look_delta: Vec2::ZERO,
        }
    }
}

/// 请求切换鼠标游标捕获状态
#[derive(Message, Debug, Clone, Copy)]
pub struct ToggleCursorCapture;

/// 收集本帧玩家输入意图
pub fn collect_player_intent(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<AccumulatedMouseMotion>,
    mut intent: ResMut<PlayerIntent>,
) {
    let x = keys.pressed(KeyCode::KeyD) as i8 - keys.pressed(KeyCode::KeyA) as i8;
    let y = keys.pressed(KeyCode::Space) as i8 - keys.pressed(KeyCode::ControlLeft) as i8;
    let z = keys.pressed(KeyCode::KeyW) as i8 - keys.pressed(KeyCode::KeyS) as i8;

    intent.move_axis = Vec3::new(x as f32, y as f32, z as f32);
    intent.sprint = keys.pressed(KeyCode::ShiftLeft);
    intent.look_delta = mouse.delta;
}

pub fn emit_cursor_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut toggle: MessageWriter<ToggleCursorCapture>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        toggle.write(ToggleCursorCapture);
    }
}
