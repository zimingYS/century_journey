pub mod stats;

use crate::game::constant::player::*;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player;

/// 本地玩家标记。
///
/// 联机远程玩家也会拥有 Player / PlayerRig，但只有本地玩家会绑定本机相机、输入和第一人称可见性。
#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component)]
pub struct PlayerMovement {
    pub movement_speed: f32,
    pub sprint_factor: f32,
    pub jump_force: f32,
    /// 地面加速度，决定起步达到目标速度所需的时间。
    pub acceleration: f32,
    /// 松开方向键后的地面减速度。
    pub deceleration: f32,
    /// 空中水平控制相对地面控制的比例。
    pub air_control: f32,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            movement_speed: 10.0,
            sprint_factor: 1.5,
            jump_force: 8.0,
            acceleration: 90.0,
            deceleration: 180.0,
            air_control: 0.2,
        }
    }
}

/// 玩家当前水平速度。垂直速度仍由 PlayerGravity 维护。
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerVelocity {
    pub horizontal: Vec3,
}

/// 玩家碰撞箱组件，描述一个轴对齐包围盒（AABB）
#[derive(Component)]
pub struct PlayerCollider {
    /// 碰撞箱半尺寸
    pub half_extents: Vec3,
}

impl Default for PlayerCollider {
    fn default() -> Self {
        // 玩家碰撞箱
        Self {
            half_extents: Vec3::new(PLAYER_HALF_WIDTH, PLAYER_HALF_HEIGHT, PLAYER_HALF_DEPTH),
        }
    }
}

// 玩家重力与着地状态组件
#[derive(Component)]
pub struct PlayerGravity {
    /// 当前垂直速度（世界单位/秒），正数向上
    pub velocity_y: f32,
    /// 玩家是否站在固体方块上
    pub is_grounded: bool,
    /// 本次离地后累计的向下位移，用于落地伤害。
    pub fall_distance: f32,
}

impl Default for PlayerGravity {
    fn default() -> Self {
        Self {
            velocity_y: 0.0,
            is_grounded: false,
            fall_distance: 0.0,
        }
    }
}

/// 玩家生存生命周期。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerLifeState {
    #[default]
    Alive,
    Dead,
    Respawning,
}

#[derive(Component, Debug, Clone)]
pub struct PlayerLifecycle {
    pub state: PlayerLifeState,
    /// Respawning 状态保留一个很短的过渡，确保状态变化可被 UI 和动画观察到。
    pub respawn_remaining: f32,
}

impl Default for PlayerLifecycle {
    fn default() -> Self {
        Self {
            state: PlayerLifeState::Alive,
            respawn_remaining: 0.0,
        }
    }
}

impl PlayerLifecycle {
    pub const fn is_alive(&self) -> bool {
        matches!(self.state, PlayerLifeState::Alive)
    }
}

/// 玩家个人重生点，会随玩家存档保存。
#[derive(Component, Debug, Clone, Copy)]
pub struct RespawnPoint(pub Vec3);

impl Default for RespawnPoint {
    fn default() -> Self {
        Self(Vec3::new(0.0, 70.0, 0.0))
    }
}

/// 环境暴露计时，集中保存溺水和周期环境伤害的状态。
#[derive(Component, Debug, Clone, Copy)]
pub struct EnvironmentExposure {
    pub remaining_air: f32,
    pub damage_cooldown: f32,
}

impl Default for EnvironmentExposure {
    fn default() -> Self {
        Self {
            remaining_air: 10.0,
            damage_cooldown: 0.0,
        }
    }
}
