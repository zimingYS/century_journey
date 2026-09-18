//! cj-game 插件

use crate::input::cursor::{lock_cursor_on_start, toggle_cursor_capture};
use crate::input::intent::{
    PlayerIntent, ToggleCursorCapture, collect_player_intent, emit_cursor_toggle,
};
use crate::sets::GameSet;
use bevy::prelude::*;

pub struct GameInputPlugin;

impl Plugin for GameInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerIntent>()
            .add_message::<ToggleCursorCapture>()
            .add_systems(Startup, lock_cursor_on_start)
            .add_systems(Update, collect_player_intent.in_set(GameSet::Input))
            .add_systems(Update, emit_cursor_toggle.in_set(GameSet::Input))
            .add_systems(Update, toggle_cursor_capture.in_set(GameSet::Interaction));
    }
}
