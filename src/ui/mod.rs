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
    fn build(&self, app: &mut App) {
        app
            // ── 消息通道 ──
            .add_message::<SlotClickedEvent>()
            .add_message::<CategoryClickedEvent>()

            // ── 资源 ──
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

            // ── Update: 数据构建 ──
            .add_systems(Update, (
                screens::creative_inventory::build_creative_categories_system,
                screens::creative_inventory::update_creative_filter_system,
            ).chain())

            // ── Update: UI填充 ──
            .add_systems(Update, (
                screens::creative_inventory::populate_creative_grid_system,
                screens::creative_inventory::populate_recent_panel_system,
                screens::creative_inventory::init_creative_hotbar_system,
            ).after(screens::creative_inventory::update_creative_filter_system))

            // ── Update: 输入 → 事件 ──
            .add_systems(Update, (
                interaction::slot_interaction_system,
                interaction::category_tab_interaction_system,
                interaction::search_box_interaction_system,
            ))

            // ── Update: 事件 → 状态 ──
            .add_systems(Update, (
                interaction::handle_slot_clicked_system,
                interaction::handle_category_clicked_system,
                interaction::cancel_drag_system,
            ))

            // ── Update: 搜索 ──
            .add_systems(Update, (
                interaction::search_keyboard_input_system,
                interaction::search_escape_system,
                interaction::update_search_text_display_system,
            ))

            // ── Update: UI 视觉同步 ──
            .add_systems(Update, (
                screens::creative_inventory::toggle_creative_inventory_system,
                screens::creative_inventory::update_creative_visibility_system,
                screens::creative_inventory::creative_hotbar_visual_sync_system,
                screens::creative_inventory::cleanup_creative_hotbar_system,
                screens::creative_inventory::update_category_highlight_system,
                interaction::slot_hover_system,
            ))

            // ── Update: HUD ──
            .add_systems(Update, (
                hud::hotbar::hud_hotbar_visual_sync_system,
                hud::hotbar::handle_hotbar_switch_system,
                hud::sync_hud_visibility_system,
                sync_input_blocked_system,
            ))

            // ── Update: 光标 ──
            .add_systems(Update, (
                widgets::drag::cursor_follow_system,
                widgets::drag::cursor_visibility_system,
                widgets::drag::cursor_texture_system,
            ));
    }
}


fn sync_input_blocked_system(
    state: Res<InventoryState>,
    mut input_blocked: ResMut<InputBlocked>,
) {
    input_blocked.0 = state.opened;
}
