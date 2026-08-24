//! 把当前帧键鼠状态采集为玩家动作，并写入下一固定步的命令缓冲区。

use bevy::ecs::system::SystemParam;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use crate::client::camera::FpsCamera;
use crate::game::player::control::action::{PlayerAction, PlayerActionState};
use crate::game::player::control::command::{PlayerCommand, PlayerCommandBuffer};
use crate::game::player::flight::components::ToggleFlightRequest;
use crate::game::player::identity::LocalPlayer;
use crate::game::save::SaveDebugCommand;
use crate::game::world::time::WorldSimulationClock;
use crate::shared::states::InputContextState;

/// 当前渲染帧采集到的本地玩家动作快照。
#[derive(Resource, Debug, Clone, Default)]
pub struct ClientActionState(PlayerActionState);

impl std::ops::Deref for ClientActionState {
    type Target = PlayerActionState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ClientActionState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// 单帧玩家动作采集所需的本机输入、只读视角和消息出口。
#[derive(SystemParam)]
pub(super) struct PlayerActionInput<'w, 's> {
    time: Res<'w, Time>,
    keyboard: Res<'w, ButtonInput<KeyCode>>,
    mouse: Res<'w, ButtonInput<MouseButton>>,
    mouse_wheel: MessageReader<'w, 's, MouseWheel>,
    context: Res<'w, InputContextState>,
    state: ResMut<'w, ClientActionState>,
    clock: Option<Res<'w, WorldSimulationClock>>,
    command_buffer: Option<ResMut<'w, PlayerCommandBuffer>>,
    player_query: Query<'w, 's, &'static Transform, With<LocalPlayer>>,
    camera_query: Query<'w, 's, &'static FpsCamera, With<Camera3d>>,
    save_debug_commands: MessageWriter<'w, SaveDebugCommand>,
    flight_requests: MessageWriter<'w, ToggleFlightRequest>,
}

/// 在渲染帧采集本地输入，并把下一模拟刻命令写入缓冲区。
pub(super) fn collect_player_actions_system(
    mut input: PlayerActionInput,
    mut last_jump: Local<f32>,
) {
    // 双击空格切换飞行：两次按下间隔小于窗口则发请求，并重置防止三连触发。
    const DOUBLE_TAP_SECONDS: f32 = 0.3;

    let mut actions = Vec::with_capacity(16);
    if input.context.active().allows_gameplay() {
        push_pressed(
            &input.keyboard,
            KeyCode::KeyW,
            PlayerAction::MoveForward,
            &mut actions,
        );

        push_pressed(
            &input.keyboard,
            KeyCode::KeyS,
            PlayerAction::MoveBackward,
            &mut actions,
        );

        push_pressed(
            &input.keyboard,
            KeyCode::KeyA,
            PlayerAction::MoveLeft,
            &mut actions,
        );

        push_pressed(
            &input.keyboard,
            KeyCode::KeyD,
            PlayerAction::MoveRight,
            &mut actions,
        );

        if input.keyboard.pressed(KeyCode::ShiftLeft) || input.keyboard.pressed(KeyCode::ShiftRight)
        {
            actions.push(PlayerAction::Sprint);
        }

        if input.keyboard.pressed(KeyCode::ControlLeft)
            || input.keyboard.pressed(KeyCode::ControlRight)
        {
            actions.push(PlayerAction::Squat);
        }

        push_pressed(
            &input.keyboard,
            KeyCode::Space,
            PlayerAction::Jump,
            &mut actions,
        );

        if input.keyboard.just_pressed(KeyCode::Space) {
            let now = input.time.elapsed_secs();
            if *last_jump >= 0.0 && now - *last_jump < DOUBLE_TAP_SECONDS {
                input.flight_requests.write(ToggleFlightRequest);
                *last_jump = -1.0;
            } else {
                *last_jump = now;
            }
        }

        if input.mouse.pressed(MouseButton::Left) {
            actions.extend([PlayerAction::BreakBlock, PlayerAction::Attack]);
        }

        if input.mouse.pressed(MouseButton::Right) {
            actions.extend([PlayerAction::PlaceBlock, PlayerAction::Use]);
        }

        if input.keyboard.just_pressed(KeyCode::KeyQ) {
            actions.push(PlayerAction::DropItem);
        }

        if input.keyboard.just_pressed(KeyCode::F9) {
            input
                .save_debug_commands
                .write(SaveDebugCommand::InspectWorldMetadata);
        }

        if input.keyboard.just_pressed(KeyCode::F5) {
            let control_pressed = input.keyboard.pressed(KeyCode::ControlLeft)
                || input.keyboard.pressed(KeyCode::ControlRight);
            if control_pressed {
                input.save_debug_commands.write(SaveDebugCommand::SaveWorld);
            } else {
                actions.push(PlayerAction::TogglePerspective);
            }
        }

        let hotbar_keys = [
            PlayerAction::Hotbar1,
            PlayerAction::Hotbar2,
            PlayerAction::Hotbar3,
            PlayerAction::Hotbar4,
            PlayerAction::Hotbar5,
            PlayerAction::Hotbar6,
            PlayerAction::Hotbar7,
            PlayerAction::Hotbar8,
            PlayerAction::Hotbar9,
        ];
        let key_codes = [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
            KeyCode::Digit6,
            KeyCode::Digit7,
            KeyCode::Digit8,
            KeyCode::Digit9,
        ];

        for (key, action) in key_codes.into_iter().zip(hotbar_keys) {
            if input.keyboard.just_pressed(key) {
                actions.push(action);
            }
        }

        for event in input.mouse_wheel.read() {
            if event.y > 0.0 {
                actions.push(PlayerAction::HotbarPrevious);
            } else if event.y < 0.0 {
                actions.push(PlayerAction::HotbarNext);
            }
        }
    } else {
        input.mouse_wheel.clear();
    }
    input
        .state
        .update(input.context.active().allows_gameplay(), actions);

    let yaw = input
        .player_query
        .single()
        .map(|transform| transform.rotation.to_euler(EulerRot::YXZ).0)
        .unwrap_or(0.0);
    let pitch = input
        .camera_query
        .single()
        .map(|camera| camera.pitch)
        .unwrap_or(0.0);
    let Some(clock) = input.clock.as_deref() else {
        return;
    };
    let command = PlayerCommand::from_action_state(
        clock.simulation_tick().saturating_add(1),
        &input.state,
        yaw,
        pitch,
    );
    if let Some(command_buffer) = input.command_buffer.as_deref_mut() {
        command_buffer.enqueue(command);
    }
}

fn push_pressed(
    keyboard: &ButtonInput<KeyCode>,
    key: KeyCode,
    action: PlayerAction,
    actions: &mut Vec<PlayerAction>,
) {
    if keyboard.pressed(key) {
        actions.push(action);
    }
}
