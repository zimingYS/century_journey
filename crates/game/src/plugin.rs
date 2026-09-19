//! cj-game 插件

use crate::input::GameInputPlugin;
use crate::player::GamePlayerPlugin;
use crate::sets::GameSet;
use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                GameSet::Input,
                GameSet::Interaction,
                GameSet::Player,
                GameSet::Domain,
                GameSet::Ui,
            )
                .chain(),
        )
        .add_plugins((GameInputPlugin, GamePlayerPlugin));
    }
}
