pub mod app_state;
pub mod inventory_ui_state;

use bevy::prelude::*;
pub struct CoreStatePlugin;

impl Plugin for CoreStatePlugin{
    fn build(&self, app: &mut App) { app
        .init_state::<app_state::AppState>()
        .init_resource::<inventory_ui_state::InventoryUiState>();
    }
}