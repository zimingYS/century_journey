use std::collections::HashSet;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::content::block::event::{BlockInteractEvent, BlockPlaceEvent};
use crate::game::gameplay::block_action::BlockBreakProgress;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::game::player::components::stats::Health;
use crate::game::player::components::{FoodUseState, LocalPlayer, PlayerGravity};
use crate::game::player::events::{DamageEvent, DeathEvent, FoodConsumedEvent};
use crate::shared::components::camera::FpsCamera;

/// 下半身移动状态。该状态只描述玩家正在怎样移动，不负责移动玩家。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerLocomotionState {
    #[default]
    Idle,
    Walk,
    Run,
    Jump,
    Fall,
}

/// 上半身行为状态。枚举值本身不代表优先级，由 priority 方法统一定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerBehaviorState {
    #[default]
    None,
    Mining,
    Placing,
    Using,
    Attacking,
    Hurt,
    Death,
}

impl PlayerBehaviorState {
    /// 行为优先级只解决动画竞争，不参与游戏规则判定。
    pub const fn priority(self) -> u8 {
        match self {
            Self::None => 0,
            Self::Using => 40,
            Self::Placing => 50,
            Self::Mining => 60,
            Self::Attacking => 70,
            Self::Hurt => 90,
            Self::Death => 100,
        }
    }

    pub const fn loops(self) -> bool {
        matches!(self, Self::Mining | Self::Using)
    }

    pub const fn marker(self) -> Option<AnimationMarkerKind> {
        match self {
            Self::Mining => Some(AnimationMarkerKind::MiningSwing),
            Self::Placing => Some(AnimationMarkerKind::PlaceCommit),
            Self::Using => Some(AnimationMarkerKind::UseCommit),
            Self::Attacking => Some(AnimationMarkerKind::AttackHit),
            Self::None | Self::Hurt | Self::Death => None,
        }
    }
}

/// 动画标记只用于声音、粒子和镜头反馈。
///
/// 游戏逻辑不能依赖这些标记完成伤害、挖掘或放置，否则低帧率会改变规则结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationMarkerKind {
    AttackHit,
    MiningSwing,
    PlaceCommit,
    UseCommit,
}

/// 动画播放到关键时间点时发出的表现事件。
#[derive(Message, Debug, Clone, Copy)]
pub struct AnimationMarkerEvent {
    pub player: Entity,
    pub behavior: PlayerBehaviorState,
    pub marker: AnimationMarkerKind,
    pub cycle: u32,
    pub normalized_time: f32,
}

/// 一层动画状态及其过渡权重。
#[derive(Debug, Clone)]
pub struct AnimationLayer<S> {
    pub current: S,
    pub previous: S,
    pub transition_elapsed: f32,
    pub transition_duration: f32,
    pub weight: f32,
    pub target_weight: f32,
}

impl<S: Copy + PartialEq> AnimationLayer<S> {
    pub fn new(state: S, weight: f32) -> Self {
        Self {
            current: state,
            previous: state,
            transition_elapsed: 0.0,
            transition_duration: 0.0,
            weight,
            target_weight: weight,
        }
    }

    /// 切换目标状态。这里只改变动画层，不会修改玩家的游戏状态。
    pub fn transition_to(&mut self, next: S, duration: f32, target_weight: f32) {
        if self.current != next {
            self.previous = self.current;
            self.current = next;
            self.transition_elapsed = 0.0;
            self.transition_duration = duration.max(0.0);
        }
        self.target_weight = target_weight.clamp(0.0, 1.0);
        if duration <= f32::EPSILON {
            self.previous = self.current;
            self.weight = self.target_weight;
        }
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        let delta_seconds = delta_seconds.max(0.0);
        self.transition_elapsed =
            (self.transition_elapsed + delta_seconds).min(self.transition_duration);
        let weight_step = if self.transition_duration <= f32::EPSILON {
            1.0
        } else {
            delta_seconds / self.transition_duration
        };
        self.weight = move_towards(self.weight, self.target_weight, weight_step);
        if self.transition_elapsed >= self.transition_duration {
            self.previous = self.current;
        }
    }

    /// 当前状态相对上一状态的平滑混合比例。
    pub fn blend_factor(&self) -> f32 {
        if self.transition_duration <= f32::EPSILON {
            return 1.0;
        }
        smoothstep((self.transition_elapsed / self.transition_duration).clamp(0.0, 1.0))
    }
}

/// 第一人称和第三人称共同消费的动作参数。
#[derive(Debug, Clone)]
pub struct PlayerAnimationParameters {
    pub horizontal_speed: f32,
    pub vertical_speed: f32,
    pub grounded: bool,
    pub holding_item: bool,
    pub locomotion_phase: f32,
    pub action_progress: f32,
    pub playback_speed: f32,
    /// 本地相机俯仰角，供第三人称头部和上身跟随视线。
    pub look_pitch: f32,
}

impl Default for PlayerAnimationParameters {
    fn default() -> Self {
        Self {
            horizontal_speed: 0.0,
            vertical_speed: 0.0,
            grounded: false,
            holding_item: false,
            locomotion_phase: 0.0,
            action_progress: 0.0,
            playback_speed: 1.0,
            look_pitch: 0.0,
        }
    }
}

/// 单个行为动画的播放游标。
#[derive(Debug, Clone, Default)]
pub struct AnimationPlayback {
    pub elapsed: f32,
    pub duration: f32,
    pub cycle: u32,
    pub active: bool,
    marker_fired: bool,
}

impl AnimationPlayback {
    fn start(&mut self, duration: f32) {
        self.elapsed = 0.0;
        self.duration = duration.max(0.0001);
        self.cycle = 0;
        self.active = true;
        self.marker_fired = false;
    }

    fn stop(&mut self) {
        self.elapsed = 0.0;
        self.duration = 0.0;
        self.active = false;
        self.marker_fired = false;
    }

    pub fn normalized_time(&self) -> f32 {
        if !self.active || self.duration <= f32::EPSILON {
            return 0.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// 推进播放游标，并返回是否结束、是否跨过标记点。
    fn tick(
        &mut self,
        delta_seconds: f32,
        speed: f32,
        looping: bool,
        marker_fraction: Option<f32>,
    ) -> PlaybackTick {
        if !self.active {
            return PlaybackTick::default();
        }

        let previous = self.elapsed;
        let marker_cycle = self.cycle;
        let raw_elapsed = previous + delta_seconds.max(0.0) * speed.max(0.0);
        let marker_time = marker_fraction.map(|fraction| self.duration * fraction.clamp(0.0, 1.0));
        let marker_crossed = marker_time
            .is_some_and(|marker| !self.marker_fired && previous < marker && raw_elapsed >= marker);
        if marker_crossed {
            self.marker_fired = true;
        }

        if looping && raw_elapsed >= self.duration {
            let completed_cycles = (raw_elapsed / self.duration).floor() as u32;
            self.cycle = self.cycle.saturating_add(completed_cycles.max(1));
            self.elapsed = raw_elapsed % self.duration;
            self.marker_fired = marker_time.is_some_and(|marker| self.elapsed >= marker);
            return PlaybackTick {
                finished: false,
                marker_crossed,
                marker_cycle,
            };
        }

        self.elapsed = raw_elapsed.min(self.duration);
        PlaybackTick {
            finished: !looping && self.elapsed >= self.duration,
            marker_crossed,
            marker_cycle,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct PlaybackTick {
    finished: bool,
    marker_crossed: bool,
    marker_cycle: u32,
}

#[derive(Debug, Clone, Copy)]
struct PendingMarker {
    behavior: PlayerBehaviorState,
    marker: AnimationMarkerKind,
    cycle: u32,
    normalized_time: f32,
}

/// 统一玩家动画组件。
///
/// 控制器写入这份组件，第一人称真实手部和第三人称模型都从这里读取参数。
#[derive(Component, Debug, Clone)]
pub struct PlayerAnimationState {
    pub lower_body: AnimationLayer<PlayerLocomotionState>,
    pub upper_body: AnimationLayer<PlayerBehaviorState>,
    pub parameters: PlayerAnimationParameters,
    pub playback: AnimationPlayback,
    pub previous_behavior_progress: f32,
    previous_position: Option<Vec3>,
    pending_marker: Option<PendingMarker>,
}

impl Default for PlayerAnimationState {
    fn default() -> Self {
        Self {
            lower_body: AnimationLayer::new(PlayerLocomotionState::Idle, 1.0),
            upper_body: AnimationLayer::new(PlayerBehaviorState::None, 0.0),
            parameters: PlayerAnimationParameters::default(),
            playback: AnimationPlayback::default(),
            previous_behavior_progress: 0.0,
            previous_position: None,
            pending_marker: None,
        }
    }
}

/// 动画速度、过渡和事件时间点集中配置，避免散落在独立动画函数中。
#[derive(Resource, Debug, Clone)]
pub struct PlayerAnimationConfig {
    pub locomotion_transition_seconds: f32,
    pub behavior_transition_seconds: f32,
    pub walk_cycle_speed: f32,
    pub run_cycle_speed: f32,
    pub mining_duration: f32,
    pub placing_duration: f32,
    pub using_duration: f32,
    pub attack_duration: f32,
    pub hurt_duration: f32,
    pub death_duration: f32,
}

impl Default for PlayerAnimationConfig {
    fn default() -> Self {
        Self {
            locomotion_transition_seconds: 0.14,
            behavior_transition_seconds: 0.08,
            walk_cycle_speed: 7.0,
            run_cycle_speed: 10.5,
            mining_duration: 0.42,
            placing_duration: 0.28,
            using_duration: 0.46,
            attack_duration: 0.36,
            hurt_duration: 0.34,
            death_duration: 1.1,
        }
    }
}

impl PlayerAnimationConfig {
    fn duration(&self, behavior: PlayerBehaviorState) -> f32 {
        match behavior {
            PlayerBehaviorState::None => 0.0,
            PlayerBehaviorState::Mining => self.mining_duration,
            PlayerBehaviorState::Placing => self.placing_duration,
            PlayerBehaviorState::Using => self.using_duration,
            PlayerBehaviorState::Attacking => self.attack_duration,
            PlayerBehaviorState::Hurt => self.hurt_duration,
            PlayerBehaviorState::Death => self.death_duration,
        }
    }

    fn marker_fraction(&self, behavior: PlayerBehaviorState) -> Option<f32> {
        match behavior {
            PlayerBehaviorState::Mining => Some(0.52),
            PlayerBehaviorState::Placing => Some(0.55),
            PlayerBehaviorState::Using => Some(0.50),
            PlayerBehaviorState::Attacking => Some(0.45),
            PlayerBehaviorState::None | PlayerBehaviorState::Hurt | PlayerBehaviorState::Death => {
                None
            }
        }
    }
}

#[derive(SystemParam)]
pub struct AnimationControllerInput<'w, 's> {
    time: Res<'w, Time>,
    actions: Res<'w, PlayerActionState>,
    inventory: Res<'w, InventoryState>,
    break_progress: Res<'w, BlockBreakProgress>,
    config: Res<'w, PlayerAnimationConfig>,
    damage_events: MessageReader<'w, 's, DamageEvent>,
    death_events: MessageReader<'w, 's, DeathEvent>,
    place_events: MessageReader<'w, 's, BlockPlaceEvent>,
    interact_events: MessageReader<'w, 's, BlockInteractEvent>,
    food_events: MessageReader<'w, 's, FoodConsumedEvent>,
}

#[derive(Debug, Clone, Copy, Default)]
struct AnimationSignals {
    died: bool,
    hurt: bool,
    mining: bool,
    placed: bool,
    used: bool,
    attacked: bool,
}

fn choose_behavior(signals: AnimationSignals) -> Option<PlayerBehaviorState> {
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
            &Transform,
            &PlayerGravity,
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

    for (entity, transform, gravity, health, food_use, mut state) in &mut query {
        update_motion_parameters(
            &mut state,
            transform.translation,
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

fn update_motion_parameters(
    state: &mut PlayerAnimationState,
    position: Vec3,
    gravity: &PlayerGravity,
    holding_item: bool,
    delta_seconds: f32,
    config: &PlayerAnimationConfig,
    actions: &PlayerActionState,
) {
    let previous = state.previous_position.replace(position);
    let sampled_speed = previous.map_or(0.0, |last| {
        Vec2::new(position.x - last.x, position.z - last.z).length() / delta_seconds
    });
    // 单帧位移容易受帧时间和台阶抬升影响，指数平滑能让步频与状态切换保持稳定。
    let response = 1.0 - (-18.0 * delta_seconds).exp();
    let horizontal_speed = state.parameters.horizontal_speed
        + (sampled_speed.clamp(0.0, 30.0) - state.parameters.horizontal_speed) * response;
    let locomotion = if !gravity.is_grounded {
        if gravity.velocity_y > 0.0 {
            PlayerLocomotionState::Jump
        } else {
            PlayerLocomotionState::Fall
        }
    } else if horizontal_speed < 0.05 {
        PlayerLocomotionState::Idle
    } else if actions.pressed(PlayerAction::Sprint) || horizontal_speed > 11.5 {
        PlayerLocomotionState::Run
    } else {
        PlayerLocomotionState::Walk
    };

    state
        .lower_body
        .transition_to(locomotion, config.locomotion_transition_seconds, 1.0);
    let cycle_speed = match locomotion {
        PlayerLocomotionState::Walk => {
            config.walk_cycle_speed * (horizontal_speed / 10.0).clamp(0.35, 1.5)
        }
        PlayerLocomotionState::Run => {
            config.run_cycle_speed * (horizontal_speed / 15.0).clamp(0.5, 1.5)
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

fn move_towards(current: f32, target: f32, max_delta: f32) -> f32 {
    if (target - current).abs() <= max_delta {
        target
    } else {
        current + (target - current).signum() * max_delta.max(0.0)
    }
}

fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feedback_fix_empty_right_click_does_not_start_an_animation() {
        assert_eq!(choose_behavior(AnimationSignals::default()), None);
    }

    #[test]
    fn feedback_fix_consumed_food_starts_using_animation() {
        assert_eq!(
            choose_behavior(AnimationSignals {
                used: true,
                ..default()
            }),
            Some(PlayerBehaviorState::Using)
        );
    }

    #[test]
    fn death_and_hurt_override_regular_actions() {
        let all_actions = AnimationSignals {
            died: true,
            hurt: true,
            mining: true,
            placed: true,
            used: true,
            attacked: true,
        };
        assert_eq!(
            choose_behavior(all_actions),
            Some(PlayerBehaviorState::Death)
        );
        assert!(PlayerBehaviorState::Hurt.priority() > PlayerBehaviorState::Attacking.priority());
    }

    #[test]
    fn layer_transition_is_smooth_and_reaches_target() {
        let mut layer = AnimationLayer::new(PlayerLocomotionState::Idle, 1.0);
        layer.transition_to(PlayerLocomotionState::Run, 0.2, 1.0);
        layer.tick(0.1);
        assert_eq!(layer.previous, PlayerLocomotionState::Idle);
        assert_eq!(layer.current, PlayerLocomotionState::Run);
        assert!((layer.blend_factor() - 0.5).abs() < 0.001);
        layer.tick(0.1);
        assert_eq!(layer.previous, PlayerLocomotionState::Run);
        assert_eq!(layer.blend_factor(), 1.0);
    }

    #[test]
    fn playback_emits_marker_once_per_cycle() {
        let mut playback = AnimationPlayback::default();
        playback.start(1.0);
        let before = playback.tick(0.4, 1.0, false, Some(0.5));
        let crossing = playback.tick(0.2, 1.0, false, Some(0.5));
        let after = playback.tick(0.2, 1.0, false, Some(0.5));
        assert!(!before.marker_crossed);
        assert!(crossing.marker_crossed);
        assert!(!after.marker_crossed);
    }

    #[test]
    fn looping_marker_keeps_the_cycle_where_it_was_crossed() {
        let mut playback = AnimationPlayback::default();
        playback.start(1.0);
        let first = playback.tick(0.6, 1.0, true, Some(0.5));
        let wrap = playback.tick(0.5, 1.0, true, Some(0.5));
        let second = playback.tick(0.5, 1.0, true, Some(0.5));

        assert!(first.marker_crossed);
        assert_eq!(first.marker_cycle, 0);
        assert!(!wrap.marker_crossed);
        assert!(second.marker_crossed);
        assert_eq!(second.marker_cycle, 1);
    }
}
