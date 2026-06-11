use crate::core::input_block::InputBlocked;
use bevy::prelude::*;
use crate::inventory::state::InventoryState;
use crate::ui::theme::category_theme::CategoryTheme;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{CategoryClickedEvent, SearchInputState, SlotClickedEvent};

pub mod components;
pub mod resources;
pub mod hud;
pub mod widgets;
pub mod screens;
pub mod theme;
pub mod interaction;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) { app
        // ── 资源 ──
        .add_message::<SlotClickedEvent>()
        .add_message::<CategoryClickedEvent>()
        .init_resource::<InventoryState>()
        .init_resource::<UiTheme>()
        .init_resource::<CategoryTheme>()
        .init_resource::<resources::ui_font::UiFont>()
        .init_resource::<SearchInputState>()

        // ── Startup ──
        .add_systems(Startup, (
            resources::ui_font::load_ui_font_system,
            hud::crosshair::setup_crosshair,
            hud::hotbar::spawn_hotbar_ui_system,
            widgets::drag::spawn_cursor_item_icon,
            screens::creative_inventory::spawn_creative_inventory_system,
        ))

        // ── Update: 数据构建链 ──
        .add_systems(Update, (
            screens::creative_inventory::build_creative_categories_system,
            screens::creative_inventory::update_creative_filter_system,
        ).chain())

        // ── Update: 显示填充 ──
        .add_systems(Update, (
            screens::creative_inventory::populate_creative_grid_system,
            screens::creative_inventory::populate_recent_panel_system,
        ).after(screens::creative_inventory::update_creative_filter_system))

        // ── Update: 交互与显示 ──
        .add_systems(Update, (
            screens::creative_inventory::toggle_creative_inventory_system,
            screens::creative_inventory::update_creative_visibility_system,
            screens::creative_inventory::update_creative_hotbar_display_system,
            screens::creative_inventory::update_category_highlight_system,
            screens::creative_inventory::handle_creative_click_system,
            // screens::creative_inventory::slot_hover_system,
            interaction::slot_interaction_system,
            interaction::category_interaction_system,
            interaction::search_box_interaction_system,
            interaction::activate_search_box_system,
            interaction::search_keyboard_input_system,
            interaction::search_escape_system,
            interaction::update_search_text_display_system,
            interaction::category_tab_interaction_system,
            interaction::handle_slot_clicked_system,
            interaction::slot_hover_system,
        ))

        // ── Update: 拖拽/HUD/输入 ──
        .add_systems(Update, (
            widgets::drag::update_cursor_icon_system,
            hud::hotbar::update_hotbar_ui_system,
            hud::hotbar::handle_hotbar_switch_system,
            hud::sync_hud_visibility_system,
            sync_input_blocked_system,
        ));
    }
}


fn sync_input_blocked_system(
    state: Res<InventoryState>,
    mut input_blocked: ResMut<InputBlocked>,
) {
    input_blocked.0 = state.opened;
}
