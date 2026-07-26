use bevy::math::Vec3;
use bevy::prelude::Component;

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
