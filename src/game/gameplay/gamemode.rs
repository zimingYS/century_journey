use bevy::prelude::*;

/// 游戏模式定义
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameMode {
    /// 生存模式
    #[default]
    Survival,

    /// 创造模式
    Creative,
}

/// 当前玩家的游戏模式
#[derive(Resource, Debug)]
pub struct PlayerGameMode {
    /// 对应的游戏模式
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
    /// 当前是否为创造模式
    pub fn is_creative(&self) -> bool {
        matches!(self.mode, GameMode::Creative)
    }

    /// 当前是否为生存模式
    pub fn is_survival(&self) -> bool {
        matches!(self.mode, GameMode::Survival)
    }
}

/// F4切换游戏模式（仅开发阶段使用）
pub fn toggle_gamemode_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    context: Res<crate::shared::states::InputContextState>,
    mut gamemode: ResMut<PlayerGameMode>,
) {
    if !context.active().allows_gameplay() || !keyboard.just_pressed(KeyCode::F4) {
        return;
    }
    gamemode.mode = match gamemode.mode {
        GameMode::Creative => GameMode::Survival,
        GameMode::Survival => GameMode::Creative,
    };
    info!("游戏模式已改变为：{:?}", gamemode.mode);
}
