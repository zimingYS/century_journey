use crate::core::input_block::InputBlocked;
use bevy::prelude::*;
use crate::inventory::events::DropItemEvent;
use crate::inventory::state::InventoryState;
use crate::ui::theme::category_theme::CategoryTheme;
use crate::ui::theme::ui_theme::UiTheme;
use crate::ui::widgets::slot::{CategoryClickedEvent, SearchInputState, SlotInteractionEvent};

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
            .add_message::<SlotInteractionEvent>()
            .add_message::<CategoryClickedEvent>()
            .add_message::<DropItemEvent>()

            // ── 资源 ──
            .init_resource::<InventoryState>()
            .init_resource::<UiTheme>()
            .init_resource::<CategoryTheme>()
            .init_resource::<resources::ui_font::UiFont>()
            .init_resource::<SearchInputState>()

            // ── Startup: HudRoot + 所有 HUD 子元素 (chain 保证顺序) ──
            .add_systems(Startup, (
                hud::hotbar::spawn_hud_root_system,
                hud::crosshair::setup_crosshair,
                hud::hotbar::spawn_hotbar_ui_system,
                hud::health_bar::spawn_health_bar,
                hud::hunger_bar::spawn_hunger_bar,
                hud::armor_bar::spawn_armor_bar,
            ).chain())
            // ── Startup: 独立元素 ──
            .add_systems(Startup, (
                resources::ui_font::load_ui_font_system,
                widgets::drag::spawn_cursor_item_icon,
                screens::creative_inventory::spawn_creative_inventory_system,
                screens::survival_inventory::spawn_survival_inventory_system,
            ))

            // ── Update: 数据构建 + UI填充 (分成两组保证 chain 在 tuple 限制内) ──
            .add_systems(Update, (
                screens::creative_inventory::build_creative_categories_system,
                screens::creative_inventory::update_creative_filter_system,
                screens::creative_inventory::populate_creative_grid_system,
                screens::creative_inventory::populate_recent_panel_system,
            ).chain())
            .add_systems(Update, (
                screens::creative_inventory::init_creative_hotbar_system,
                screens::survival_inventory::populate_survival_grid_system,
                screens::survival_inventory::init_survival_hotbar_system,
            ).chain())

            // ── Update: 输入 → 事件 ──
            .add_systems(Update, (
                interaction::slot_interaction_system,
                interaction::slot_right_click_system,
                interaction::slot_q_drop_system,
                interaction::category_tab_interaction_system,
                interaction::search_box_interaction_system,
            ))

            // ── Update: 事件 → 状态 ──
            .add_systems(Update, (
                interaction::handle_slot_interaction_system,
                interaction::handle_category_clicked_system,
                interaction::cancel_drag_system,
            ))

            // ── Update: 搜索 ──
            .add_systems(Update, (
                interaction::search_keyboard_input_system,
                interaction::search_escape_system,
                interaction::update_search_text_display_system,
            ))

            // ── Update: UI 视觉同步 — toggle → visibility → sync → cleanup ──
            .add_systems(Update, (
                screens::creative_inventory::toggle_inventory_system,
                screens::creative_inventory::update_creative_visibility_system,
                screens::survival_inventory::update_survival_visibility_system,
            ))
            .add_systems(Update, (
                screens::creative_inventory::creative_hotbar_visual_sync_system,
                screens::survival_inventory::survival_hotbar_visual_sync_system,
                screens::survival_inventory::survival_grid_visual_sync_system,
                screens::creative_inventory::update_category_highlight_system,
            ).chain())
            .add_systems(Update, (
                screens::creative_inventory::cleanup_creative_hotbar_system,
                screens::survival_inventory::cleanup_survival_hotbar_system,
                interaction::slot_hover_system,
            ).chain())

            // ── Update: HUD ──
            .add_systems(Update, (
                hud::sync_hud_visibility_system,
                hud::hotbar::hud_hotbar_visual_sync_system,
                hud::hotbar::handle_hotbar_switch_system,
                hud::health_bar::health_bar_sync_system,
            ))
            .add_systems(Update, (
                hud::hunger_bar::hunger_bar_sync_system,
                hud::armor_bar::armor_bar_sync_system,
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
