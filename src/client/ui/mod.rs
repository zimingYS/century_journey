use crate::client::ui::hud::plugin::HudPlugin;
use crate::client::ui::theme::category_theme::CategoryTheme;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    CategoryClickedEvent, SearchInputState, SlotInteractionEvent,
};
use crate::game::inventory::events::DropItemEvent;
use crate::game::inventory::state::InventoryState;
use crate::shared::states::input_blocked::InputBlocked;
use bevy::prelude::*;

pub mod components;
pub mod hud;
pub mod interaction;
pub mod resources;
pub mod screens;
pub mod theme;
pub mod widgets;

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
            .init_resource::<crate::game::crafting::grid::PlayerCrafting>()
            .init_resource::<crate::game::inventory::equipment::AccessorySlotDefinitions>()
            .init_resource::<UiTheme>()
            .init_resource::<CategoryTheme>()
            .init_resource::<resources::ui_font::UiFont>()
            .init_resource::<SearchInputState>()
            .add_plugins(HudPlugin)
            // ── Startup: 独立元素 ──
            .add_systems(
                Startup,
                (
                    resources::ui_font::load_ui_font_system,
                    widgets::drag::spawn_cursor_item_icon,
                    screens::creative_inventory::spawn_creative_inventory_system,
                    screens::survival_inventory::spawn_survival_inventory_system,
                    screens::crafting::spawn_crafting_system,
                )
                    .chain(),
            )
            // ── Update: 数据构建 + UI填充 (分成两组保证 chain 在 tuple 限制内) ──
            .add_systems(
                Update,
                (
                    screens::creative_inventory::build_creative_categories_system,
                    screens::creative_inventory::update_creative_filter_system,
                    screens::creative_inventory::populate_creative_grid_system,
                    screens::creative_inventory::populate_recent_panel_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    screens::survival_inventory::sync_accessory_slot_count_system,
                    screens::creative_inventory::init_creative_hotbar_system,
                    screens::survival_inventory::populate_survival_grid_system,
                    screens::survival_inventory::init_survival_hotbar_system,
                )
                    .chain(),
            )
            // ── Update: 输入 → 事件 ──
            .add_systems(Update, interaction::active_hotbar_q_drop_system)
            .add_systems(
                Update,
                (
                    interaction::slot_interaction_system,
                    interaction::slot_right_click_system,
                    interaction::slot_q_drop_system,
                    interaction::category_tab_interaction_system,
                )
                    .run_if(|state: Res<InventoryState>| state.opened),
            )
            // ── Update: 事件 → 状态 ──
            .add_systems(
                Update,
                (
                    interaction::handle_slot_interaction_system,
                    screens::crafting::crafting_interaction_system,
                    interaction::handle_category_clicked_system,
                    interaction::cancel_drag_system,
                )
                    .run_if(|state: Res<InventoryState>| state.opened),
            )
            // ── Update: 搜索 ──
            .add_systems(
                Update,
                (
                    interaction::sync_search_input_focus_system,
                    interaction::sync_search_text_from_editable_system,
                    interaction::search_escape_system,
                )
                    .chain(),
            )
            // ── Update: UI 视觉同步 — toggle → visibility → sync → cleanup ──
            .add_systems(
                Update,
                (
                    screens::creative_inventory::toggle_inventory_system,
                    screens::creative_inventory::creative_close_button_system,
                    screens::creative_inventory::update_creative_visibility_system,
                    screens::survival_inventory::update_survival_visibility_system,
                ),
            )
            .add_systems(
                Update,
                (
                    screens::creative_inventory::creative_hotbar_visual_sync_system,
                    screens::survival_inventory::survival_hotbar_visual_sync_system,
                    screens::survival_inventory::survival_grid_visual_sync_system,
                    screens::survival_inventory::survival_stats_visual_sync_system,
                    screens::crafting::crafting_visual_sync_system,
                    screens::creative_inventory::update_category_highlight_system,
                    screens::creative_inventory::sync_creative_search_placeholder_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    screens::creative_inventory::cleanup_creative_hotbar_system,
                    screens::survival_inventory::cleanup_survival_hotbar_system,
                    interaction::slot_hover_system,
                    screens::survival_inventory::backpack_management_button_system,
                    screens::crafting::return_crafting_on_close_system,
                )
                    .chain(),
            )
            .add_systems(Update, (sync_input_blocked_system,))
            // ── Update: 光标 ──
            .add_systems(
                Update,
                (
                    widgets::drag::cursor_follow_system,
                    widgets::drag::cursor_visibility_system,
                    widgets::drag::cursor_texture_system,
                ),
            );
    }
}

fn sync_input_blocked_system(state: Res<InventoryState>, mut input_blocked: ResMut<InputBlocked>) {
    input_blocked.0 = state.opened;
}
