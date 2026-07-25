//! 将玩家输入、游戏事件和移动状态转换为动画状态。
//!
//! 这里仅负责表现层状态机，不直接修改生命值、背包或方块世界；实际游戏规则
//! 通过事件和已有组件提供输入，避免动画帧率影响模拟结果。

use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::content::block::event::{BlockInteractEvent, BlockPlaceEvent};
use crate::game::gameplay::block_action::BlockBreakProgress;
use crate::game::inventory::state::LocalInventory;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::stats::Health;
use crate::game::player::components::{FoodUseState, LocalPlayer, PlayerGravity, PlayerVelocity};
use crate::game::player::events::{DamageEvent, DeathEvent, FoodConsumedEvent};
use crate::shared::components::camera::FpsCamera;

use super::*;
#[derive(SystemParam)]
/// 动画控制系统在单帧内读取的游戏状态和事件集合。
///
/// 参数保持只读；动画系统不得通过该入口反向修改权威模拟状态。
pub struct AnimationControllerInput<'w, 's> {
    time: Res<'w, Time>,
    actions: Res<'w, PlayerActionState>,
    inventory: LocalInventory<'w, 's>,
    break_progress: Res<'w, BlockBreakProgress>,
    config: Res<'w, PlayerAnimationConfig>,
    damage_events: MessageReader<'w, 's, DamageEvent>,
    death_events: MessageReader<'w, 's, DeathEvent>,
    place_events: MessageReader<'w, 's, BlockPlaceEvent>,
    interact_events: MessageReader<'w, 's, BlockInteractEvent>,
    food_events: MessageReader<'w, 's, FoodConsumedEvent>,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct AnimationSignals {
    pub(crate) died: bool,
    pub(crate) hurt: bool,
    pub(crate) mining: bool,
    pub(crate) placed: bool,
    pub(crate) used: bool,
    pub(crate) attacked: bool,
}

pub(crate) fn choose_behavior(signals: AnimationSignals) -> Option<PlayerBehaviorState> {
    if signals.died {
        Some(PlayerBehaviorState::Death)
    } else if signals.hurt {
        Some(PlayerBehaviorState::Hurt)
    } else if signals.attacked {
        Some(PlayerBehaviorState::Attacking)
    } else if signals.mining {
        Some(PlayerBehaviorState::Mining)
    } else if signals.placed {
        Some(PlayerBehaviorState::Placing)
    } else if signals.used {
        Some(PlayerBehaviorState::Using)
    } else {
        None
    }
}

/// 把游戏状态采样为动画状态。该系统绝不写回移动、生命或方块逻辑。
pub fn player_animation_controller_system(
    mut input: AnimationControllerInput,
    camera_query: Query<&FpsCamera, With<Camera3d>>,
    mut query: Query<
        (
            Entity,
            &PlayerGravity,
            &PlayerVelocity,
            &Health,
            &FoodUseState,
            &mut PlayerAnimationState,
        ),
        With<LocalPlayer>,
    >,
) {
    let delta_seconds = input.time.delta_secs().max(0.0001);
    let look_pitch = camera_query
        .single()
        .map(|camera| camera.pitch)
        .unwrap_or(0.0);
    let damaged: HashSet<Entity> = input
        .damage_events
        .read()
        .map(|event| event.target)
        .collect();
    let died: HashSet<Entity> = input
        .death_events
        .read()
        .map(|event| event.entity)
        .collect();
    let placed: HashSet<Entity> = input
        .place_events
        .read()
        .filter_map(|event| event.placer)
        .collect();
    let used: HashSet<Entity> = input
        .interact_events
        .read()
        .filter_map(|event| event.interactor)
        .collect();
    let consumed: HashSet<Entity> = input.food_events.read().map(|event| event.player).collect();
    let holding_item = !input.inventory.hotbar.active_item().is_air();

    for (entity, gravity, velocity, health, food_use, mut state) in &mut query {
        update_motion_parameters(
            &mut state,
            velocity.horizontal.length(),
            gravity,
            holding_item,
            delta_seconds,
            &input.config,
            &input.actions,
        );
        state.parameters.look_pitch = look_pitch;

        let current_behavior = state.upper_body.current;
        let playback_speed = state.parameters.playback_speed;
        let playback_tick = state.playback.tick(
            delta_seconds,
            playback_speed,
            current_behavior.loops(),
            input.config.marker_fraction(current_behavior),
        );
        state.parameters.action_progress = state.playback.normalized_time();

        if playback_tick.marker_crossed
            && let Some(marker) = current_behavior.marker()
        {
            state.pending_marker = Some(PendingMarker {
                behavior: current_behavior,
                marker,
                cycle: playback_tick.marker_cycle,
                normalized_time: state.parameters.action_progress,
            });
        }

        let signals = AnimationSignals {
            died: died.contains(&entity) || health.is_dead(),
            hurt: damaged.contains(&entity),
            mining: input.actions.pressed(PlayerAction::BreakBlock) && input.break_progress.visible,
            placed: placed.contains(&entity),
            used: food_use.is_active() || used.contains(&entity) || consumed.contains(&entity),
            attacked: input.actions.just_pressed(PlayerAction::Attack)
                && !input.break_progress.visible,
        };

        update_behavior(
            &mut state,
            choose_behavior(signals),
            playback_tick.finished,
            &input.config,
        );
        state.lower_body.tick(delta_seconds);
        state.upper_body.tick(delta_seconds);
    }
}

pub(crate) fn update_motion_parameters(
    state: &mut PlayerAnimationState,
    simulated_horizontal_speed: f32,
    gravity: &PlayerGravity,
    holding_item: bool,
    delta_seconds: f32,
    config: &PlayerAnimationConfig,
    actions: &PlayerActionState,
) {
    // The velocity is authored by the fixed-step simulation. Sampling Transform here
    // would turn each fixed tick into one large render-frame speed spike followed by
    // several zero-speed frames, which makes locomotion visibly stutter.
    let sampled_speed = simulated_horizontal_speed.max(0.0);
    let response = 1.0 - (-18.0 * delta_seconds).exp();
    let horizontal_speed = state.parameters.horizontal_speed
        + (sampled_speed.clamp(0.0, 30.0) - state.parameters.horizontal_speed) * response;
    let locomotion = if !gravity.is_grounded {
        if gravity.velocity_y > 0.0 {
            PlayerLocomotionState::Jump
        } else {
            PlayerLocomotionState::Fall
        }
    } else if sampled_speed < 0.05 {
        PlayerLocomotionState::Idle
    } else if actions.pressed(PlayerAction::Sprint) || sampled_speed > 11.5 {
        PlayerLocomotionState::Run
    } else {
        PlayerLocomotionState::Walk
    };

    state
        .lower_body
        .transition_to(locomotion, config.locomotion_transition_seconds, 1.0);
    let cycle_speed = match locomotion {
        PlayerLocomotionState::Walk => {
            config.walk_cycle_speed * (sampled_speed / 10.0).clamp(0.35, 1.5)
        }
        PlayerLocomotionState::Run => {
            config.run_cycle_speed * (sampled_speed / 15.0).clamp(0.5, 1.5)
        }
        PlayerLocomotionState::Idle | PlayerLocomotionState::Jump | PlayerLocomotionState::Fall => {
            1.5
        }
    };
    state.parameters.locomotion_phase += delta_seconds * cycle_speed;
    state.parameters.horizontal_speed = horizontal_speed;
    state.parameters.vertical_speed = gravity.velocity_y;
    state.parameters.grounded = gravity.is_grounded;
    state.parameters.holding_item = holding_item;
}

fn update_behavior(
    state: &mut PlayerAnimationState,
    requested: Option<PlayerBehaviorState>,
    current_finished: bool,
    config: &PlayerAnimationConfig,
) {
    let current = state.upper_body.current;
    if let Some(next) = requested {
        let can_interrupt =
            current_finished || next.priority() >= current.priority() || current == next;
        let should_restart = current != next || (!next.loops() && can_interrupt);
        if can_interrupt && should_restart {
            start_behavior(state, next, config);
        }
        return;
    }

    if current.loops() || current_finished {
        start_behavior(state, PlayerBehaviorState::None, config);
    }
}

fn start_behavior(
    state: &mut PlayerAnimationState,
    behavior: PlayerBehaviorState,
    config: &PlayerAnimationConfig,
) {
    state.previous_behavior_progress = state.playback.normalized_time();
    let target_weight = if behavior == PlayerBehaviorState::None {
        0.0
    } else {
        1.0
    };
    state
        .upper_body
        .transition_to(behavior, config.behavior_transition_seconds, target_weight);
    if behavior == PlayerBehaviorState::None {
        state.playback.stop();
        state.parameters.action_progress = 0.0;
    } else {
        state.playback.start(config.duration(behavior));
        state.parameters.action_progress = 0.0;
    }
}

/// 将待发送标记转换成 Bevy 消息。这里只有表现层可以订阅。
pub fn emit_animation_marker_system(
    mut query: Query<(Entity, &mut PlayerAnimationState), With<LocalPlayer>>,
    mut writer: MessageWriter<AnimationMarkerEvent>,
) {
    for (entity, mut state) in &mut query {
        let Some(pending) = state.pending_marker.take() else {
            continue;
        };
        writer.write(AnimationMarkerEvent {
            player: entity,
            behavior: pending.behavior,
            marker: pending.marker,
            cycle: pending.cycle,
            normalized_time: pending.normalized_time,
        });
    }
}
