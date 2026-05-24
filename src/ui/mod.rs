use bevy::prelude::*;
use crate::ui::hud::crosshair::setup_crosshair;

pub mod components;
pub mod resources;
pub mod hud;
pub mod menu;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup,setup_crosshair);
    }
}