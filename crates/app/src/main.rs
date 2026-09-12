//! CenturyJourney —— 程序入口

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(cj_runtime::RuntimePlugin)
        .add_plugins(cj_render::RenderPlugin)
        .add_plugins(cj_game::GamePlugin)
        .run();
}
