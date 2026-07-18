use crate::client::ui::hud::plugin::HudPlugin;
use crate::client::ui::navigation::{UiNavigation, UiScreenStack};
use crate::client::ui::theme::category_theme::CategoryTheme;
use crate::client::ui::theme::scale::UiScaleSettings;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{CategoryClickedEvent, SearchInputState};
use crate::game::inventory::state::InventoryState;
use crate::shared::states::{InputContext, InputContextState};
use bevy::prelude::*;

pub mod components;
pub mod hud;
pub mod interaction;
pub mod navigation;
pub mod resources;
pub mod screens;
pub mod screenshot_check;
pub mod theme;
pub mod widgets;

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        screenshot_check::configure_ui_screenshot_check(app);
        screens::menu::init_menu_resources(app);
        app
            // ── 消息通道 ──
            .add_message::<CategoryClickedEvent>()
            .add_message::<UiNavigation>()
            // ── 资源 ──
            .init_resource::<UiTheme>()
            .init_resource::<UiScreenStack>()
            .init_resource::<UiScaleSettings>()
            .init_resource::<CategoryTheme>()
            .init_resource::<resources::ui_font::UiFont>()
            .init_resource::<SearchInputState>()
            .init_resource::<interaction::SlotDragState>()
            .add_plugins(HudPlugin)
            // ── Startup: 独立元素 ──
            .add_systems(
                Startup,
                (
                    resources::ui_font::load_ui_font_system,
                    resources::frame_assets::create_ui_frame_assets_system,
                    widgets::drag::spawn_cursor_item_icon,
                    widgets::tooltip::spawn_item_tooltip_system,
                    screens::creative_inventory::spawn_creative_inventory_system,
                    screens::survival_inventory::spawn_survival_inventory_system,
                    screens::crafting::spawn_crafting_system,
                    screens::death::spawn_death_screen_system,
                    screens::menu::spawn_menu_screens_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    screens::menu::sync_flow_screen_stack_system,
                    screens::menu::sync_loading_text_system,
                    screens::menu::sync_dialog_text_system,
                    screens::menu::populate_world_list_system,
                    screens::menu::sync_world_name_draft_system,
                    screens::menu::sync_setting_values_system,
                    screens::menu::menu_button_system,
                    screens::death::sync_death_screen_system,
                    screens::death::respawn_button_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    screens::crafting::open_crafting_station_ui_system,
                    navigation::handle_ui_navigation_system,
                    navigation::sync_legacy_screen_state_system,
                    navigation::sync_screen_visibility_system,
                    screens::menu::sync_menu_visibility_system,
                )
                    .chain()
                    .after(screens::menu::sync_flow_screen_stack_system),
            )
            .add_systems(
                Update,
                (
                    theme::scale::sync_ui_scale_system,
                    resources::frame_assets::apply_ui_frame_system,
                    widgets::common::themed_control_interaction_system,
                    widgets::common::scroll_area_wheel_system,
                ),
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
            .add_systems(
                Update,
                (
                    interaction::slot_interaction_system,
                    interaction::slot_right_click_system,
                    interaction::slot_drag_interaction_system,
                    interaction::slot_wheel_interaction_system,
                    interaction::slot_q_drop_system,
                    interaction::category_tab_interaction_system,
                )
                    .run_if(|context: Res<InputContextState>| {
                        context.active() == InputContext::Inventory
                    }),
            )
            // ── Update: 事件 → 状态 ──
            .add_systems(
                Update,
                (interaction::handle_category_clicked_system,)
                    .run_if(|state: Res<InventoryState>| state.opened),
            )
            // ── Update: 搜索 ──
            .add_systems(
                Update,
                (
                    interaction::sync_search_input_focus_system,
                    interaction::sync_search_text_from_editable_system,
                )
                    .chain(),
            )
            // ── Update: UI 视觉同步 — toggle → visibility → sync → cleanup ──
            .add_systems(
                Update,
                (
                    screens::creative_inventory::creative_close_button_system,
                    screens::survival_inventory::update_survival_visibility_system,
                ),
            )
            .add_systems(
                Update,
                (
                    screens::crafting::sync_crafting_panel_system,
                    screens::creative_inventory::creative_hotbar_visual_sync_system,
                    screens::survival_inventory::survival_hotbar_visual_sync_system,
                    screens::survival_inventory::survival_grid_visual_sync_system,
                    screens::survival_inventory::survival_stats_visual_sync_system,
                    screens::crafting::crafting_visual_sync_system,
                    widgets::slot::sync_slot_durability_system,
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
                    widgets::tooltip::item_tooltip_system,
                )
                    .chain(),
            )
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
