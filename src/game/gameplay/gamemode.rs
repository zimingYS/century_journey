//! 定义权威游戏模式状态。

use bevy::prelude::*;

/// 游戏模式定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    /// 生存模式。
    #[default]
    Survival,
    /// 创造模式。
    Creative,
}

/// 当前玩家的权威游戏模式。
#[derive(Resource, Debug)]
pub struct PlayerGameMode {
    /// 当前生效的游戏模式。
    pub mode: GameMode,
}

impl Default for PlayerGameMode {
    fn default() -> Self {
        Self {
            mode: GameMode::Survival,
        }
    }
}

impl PlayerGameMode {
    /// 当前是否为创造模式。
    pub fn is_creative(&self) -> bool {
        matches!(self.mode, GameMode::Creative)
    }

    /// 当前是否为生存模式。
    pub fn is_survival(&self) -> bool {
        matches!(self.mode, GameMode::Survival)
    }
}
