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
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            movement_speed: 10.0,
            sprint_factor: 1.5,
            jump_force: 8.0,
        }
    }
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
}

impl Default for PlayerGravity {
    fn default() -> Self {
        Self {
            velocity_y: 0.0,
            is_grounded: false,
        }
    }
}
