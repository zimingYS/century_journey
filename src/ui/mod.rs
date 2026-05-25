use bevy::prelude::*;

pub mod components;
pub mod resources;
pub mod hud;
pub mod menu;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) { app
        .init_resource::<resources::inventory_ui_state::InventoryUiState>()
        .add_systems(Startup,(
            hud::crosshair::setup_crosshair,
            hud::hotbar::spawn_hotbar_ui_system,
        ))
        .add_systems(Update, (
            hud::hotbar:: update_hotbar_ui_system,
            hud::hotbar::handle_hotbar_switch_system,
            menu::inventory::toggle_inventory_system,
            menu::inventory::palette_click_system,
        ));
    }
}