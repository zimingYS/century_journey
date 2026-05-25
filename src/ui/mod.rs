use bevy::prelude::*;
use crate::ui::hud::crosshair::setup_crosshair;

pub mod components;
pub mod resources;
pub mod hud;
pub mod menu;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) { app
        .init_resource::<resources::inventory_ui_state::InventoryUiState>()
        .add_systems(Startup,(
            setup_crosshair,
        ))
        .add_systems(Update, (
            menu::inventory::toggle_inventory_system,
            menu::inventory::palette_click_system,
        ));
    }
}