//! cj-game 插件

use crate::player::camera::{apply_look, spawn_player_camera};
use crate::player::motion::apply_fly_motion;
use crate::sets::GameSet;
use bevy::prelude::*;

pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_camera).add_systems(
            Update,
            (apply_look, apply_fly_motion)
                .chain()
                .in_set(GameSet::Player),
        );
    }
}
