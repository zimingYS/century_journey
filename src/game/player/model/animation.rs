//! 玩家动画的状态、播放参数和时序模型。
//!
//! 控制器实现位于 [`animation_controller`]，本模块只保留可复用的数据模型，
//! 让第一人称、第三人称和表现层共享同一套动画状态。

use bevy::prelude::*;

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
        let raw_elapsed = previous + delta_seconds.max(0.0) * speed.max(0.0);
        let marker_time = marker_fraction.map(|fraction| self.duration * fraction.clamp(0.0, 1.0));
        let marker_cycle_offset = marker_time.and_then(|marker| {
            if raw_elapsed < marker {
                return None;
            }
            let offset = ((raw_elapsed - marker) / self.duration).floor() as u32;
            (offset > 0 || (!self.marker_fired && previous < marker)).then_some(offset)
        });
        let marker_crossed = marker_cycle_offset.is_some();
        let marker_cycle = self
            .cycle
            .saturating_add(marker_cycle_offset.unwrap_or_default());
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

mod animation_controller;

#[cfg(test)]
pub(crate) use animation_controller::{
    AnimationSignals, choose_behavior, update_motion_parameters,
};
pub use animation_controller::{emit_animation_marker_system, player_animation_controller_system};
#[cfg(test)]
#[path = "../../../../tests/unit/game/player/model/animation.rs"]
mod tests;
