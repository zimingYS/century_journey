//! 定义玩家瞄准方向和水平移动状态组件。

use bevy::math::Vec3;
use bevy::prelude::Component;

/// 玩家当前权威瞄准俯仰角。
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct PlayerAim {
    /// 绕局部 X 轴的俯仰弧度。
    pub pitch: f32,
}

/// 控制玩家水平移动响应和跳跃力度的规则参数。
#[derive(Component)]
pub struct PlayerMovement {
    /// 基础地面移动速度。
    pub movement_speed: f32,
    /// 冲刺时应用到基础速度的倍率。
    pub sprint_factor: f32,
    /// 起跳时设置的初始向上速度。
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
    /// 世界空间水平速度；Y 分量始终为零。
    pub horizontal: Vec3,
}
