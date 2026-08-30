//! 注册菜单、物品栏、合成和死亡界面的生命周期与视觉同步系统。

use bevy::prelude::*;

use super::{crafting, creative_inventory, death, menu, survival_inventory};
use crate::client::ui::{interaction, localization, navigation, resources, widgets};

/// 组装所有完整屏幕及其数据投影系统。
pub struct UiScreensPlugin;

impl Plugin for UiScreensPlugin {
    fn build(&self, app: &mut App) {
        menu::init_menu_resources(app);
        app.init_resource::<creative_inventory::CreativeTabPage>()
            .add_systems(
                Startup,
                (
                    resources::ui_font::load_ui_font_system,
                    resources::frame_assets::create_ui_frame_assets_system,
                    resources::creative_assets::load_creative_ui_assets_system,
                    resources::survival_assets::load_survival_ui_assets_system,
                    widgets::drag::spawn_cursor_item_icon,
                    widgets::tooltip::spawn_item_tooltip_system,
                    creative_inventory::spawn_creative_inventory_system,
                    survival_inventory::spawn_survival_inventory_system,
                    crafting::spawn_crafting_system,
                    death::spawn_death_screen_system,
                    menu::spawn_menu_screens_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    menu::sync_flow_screen_stack_system,
                    menu::sync_loading_text_system,
                    menu::sync_dialog_text_system,
                    menu::populate_world_list_system,
                    menu::sync_world_name_draft_system,
                    menu::sync_setting_values_system,
                    localization::refresh_localized_text_system,
                    menu::menu_button_system,
                    menu::keybind_ui_system,
                    menu::reset_keybind_listening_system,
                    menu::sync_keybinds_search_system,
                    menu::sync_settings_tabs_system,
                    menu::populate_keybind_list_system,
                    death::sync_death_screen_system,
                    death::respawn_button_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    crafting::open_crafting_station_ui_system,
                    navigation::handle_ui_navigation_system,
                    navigation::project_navigation_stack_from_authoritative_state_system,
                    navigation::sync_screen_visibility_system,
                    menu::sync_menu_visibility_system,
                )
                    .chain()
                    .after(menu::sync_flow_screen_stack_system),
            )
            .add_systems(
                Update,
                (
                    creative_inventory::build_creative_categories_system,
                    creative_inventory::update_creative_filter_system,
                    creative_inventory::populate_creative_grid_system,
                    creative_inventory::populate_recent_panel_system,
                    creative_inventory::creative_tab_pager_click_system,
                    creative_inventory::update_pager_button_highlight_system,
                    creative_inventory::render_creative_tabs_system,
                    creative_inventory::sync_creative_tab_pager_text_system,
                    creative_inventory::apply_creative_tab_icon_system,
                    creative_inventory::apply_creative_title_icon_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    survival_inventory::sync_accessory_slot_count_system,
                    creative_inventory::init_creative_hotbar_system,
                    survival_inventory::populate_survival_grid_system,
                    survival_inventory::init_survival_hotbar_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    creative_inventory::creative_close_button_system,
                    survival_inventory::update_survival_visibility_system,
                ),
            )
            .add_systems(
                Update,
                (
                    crafting::sync_crafting_panel_system,
                    creative_inventory::creative_hotbar_visual_sync_system,
                    survival_inventory::survival_hotbar_visual_sync_system,
                    survival_inventory::survival_grid_visual_sync_system,
                    survival_inventory::survival_stats_visual_sync_system,
                    crafting::crafting_visual_sync_system,
                    widgets::slot::sync_slot_durability_system,
                    creative_inventory::update_category_highlight_system,
                    creative_inventory::sync_creative_search_placeholder_system,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    creative_inventory::cleanup_creative_hotbar_system,
                    interaction::slot_hover_system,
                    survival_inventory::backpack_management_button_system,
                    survival_inventory::survival_close_button_system,
                    widgets::tooltip::item_tooltip_system,
                )
                    .chain(),
            );
    }
}
