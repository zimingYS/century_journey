//! 定义权威游戏模式状态及其切换命令。

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

/// 请求在生存与创造模式之间切换一次。
///
/// Client 负责把开发快捷键转换成该消息，Game 只在固定步修改权威状态。
#[derive(Message, Debug, Clone, Copy)]
pub struct ToggleGameModeRequest;

/// 在固定步中顺序消费游戏模式切换请求。
pub fn toggle_gamemode_system(
    mut requests: MessageReader<ToggleGameModeRequest>,
    mut gamemode: ResMut<PlayerGameMode>,
) {
    for _ in requests.read() {
        gamemode.mode = match gamemode.mode {
            GameMode::Creative => GameMode::Survival,
            GameMode::Survival => GameMode::Creative,
        };
        info!("游戏模式已改变为：{:?}", gamemode.mode);
    }
}
