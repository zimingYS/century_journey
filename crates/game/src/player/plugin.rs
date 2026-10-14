//! cj-game 插件

use crate::player::camera::{apply_look, spawn_player};
use crate::player::motion::{apply_fly_motion, toggle_movement_mode};
use crate::sets::GameSet;
use bevy::prelude::*;

pub struct GamePlayerPlugin;

impl Plugin for GamePlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (toggle_movement_mode, apply_look, apply_fly_motion)
                .chain()
                .in_set(GameSet::Player),
        );
    }
}
