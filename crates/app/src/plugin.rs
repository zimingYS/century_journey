//! 顶层组装：按层依次装配

use crate::bootstrap::BootstrapPlugin;
use bevy::prelude::*;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins,
            cj_runtime::RuntimePlugin,
            cj_render::RenderPlugin,
            cj_game::GamePlugin,
            BootstrapPlugin,
        ));
    }
}
