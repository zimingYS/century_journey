use bevy::prelude::*;

/// 游戏全局生命周期状态
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    InGame,
}

pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
    }
}
