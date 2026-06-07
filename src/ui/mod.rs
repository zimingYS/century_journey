use bevy::prelude::*;
use crate::core::input_block::InputBlocked;
use crate::core::state::app_state::AppState;
use crate::core::state::inventory_ui_state::InventoryUiState;
use crate::ui::menu::inventory::category_changed_system;

pub mod components;
pub mod resources;
pub mod hud;
pub mod menu;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) { app
        .add_plugins(MeshPickingPlugin)
        .add_systems(OnEnter(AppState::InGame),
            menu::inventory::init_inventory_ui_system,
        )
        .add_systems(Startup,(
            hud::crosshair::setup_crosshair,
            hud::hotbar::spawn_hotbar_ui_system,
        ))
        .add_systems(Update, category_changed_system.run_if(resource_changed::<InventoryUiState>))
        .add_systems(Update, (
            hud::hotbar:: update_hotbar_ui_system,
            hud::hotbar::handle_hotbar_switch_system,
            menu::inventory::toggle_inventory_system,
            sync_input_blocked_system,
        ));
    }
}

fn sync_input_blocked_system(
    inventory_ui_state: Res<InventoryUiState>,
    mut input_blocked: ResMut<InputBlocked>,
) {
    input_blocked.0 = inventory_ui_state.is_inventory_open;
}