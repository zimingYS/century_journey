use bevy::prelude::*;

/// 游戏全局生命周期状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Boot,
    Loading,
    MainMenu,
    WorldLoading,
    InGame,
    Paused,
}
