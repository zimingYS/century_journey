use bevy::prelude::*;
pub mod block_action;
pub mod gamemode;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<gamemode::PlayerGameMode>()
            .add_systems(Update, gamemode::toggle_gamemode_system);
    }
}
