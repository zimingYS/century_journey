use crate::game::constant::{PLAYER_HALF_DEPTH, PLAYER_HALF_HEIGHT, PLAYER_HALF_WIDTH};
use bevy::math::Vec3;
use bevy::prelude::Component;

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
