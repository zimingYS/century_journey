//! 定义玩家存活状态、重生点和生命周期组件。

use bevy::math::Vec3;
use bevy::prelude::Component;

/// 玩家生存生命周期。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayerLifeState {
    /// 玩家存活且可参与玩法规则。
    #[default]
    Alive,
    /// 玩家已死亡，等待重生请求。
    Dead,
    /// 已恢复数据但仍保留一小段可观察的重生过渡。
    Respawning,
}

#[derive(Component, Debug, Clone)]
/// 保存玩家当前生命阶段和重生过渡剩余时间。
pub struct PlayerLifecycle {
    /// 当前权威生命阶段。
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
    /// 判断玩家是否处于允许正常玩法系统运行的存活阶段。
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
