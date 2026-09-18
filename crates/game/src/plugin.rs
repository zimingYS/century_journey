//! cj-game 插件

use crate::input::GameInputPlugin;
use crate::player::GamePlayerPlugin;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(GameInputPlugin)
            .add_plugins(GamePlayerPlugin);
    }
}
