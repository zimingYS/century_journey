//! 组装创造物品栏的分类、网格、快捷栏和界面交互。

mod catalog;
mod grid;
mod hotbar;
mod interaction;
mod skin;

pub use crate::client::ui::screens::setup::spawn_creative_inventory_system;
pub use catalog::{
    build_creative_categories_system, render_creative_tabs_system,
    sync_creative_tab_pager_text_system, update_creative_filter_system,
};
pub use grid::{populate_creative_grid_system, populate_recent_panel_system};
pub use hotbar::{
    cleanup_creative_hotbar_system, creative_hotbar_visual_sync_system, init_creative_hotbar_system,
};
pub use interaction::{
    CreativeTabPage, creative_close_button_system, creative_tab_pager_click_system,
    sync_creative_search_placeholder_system, update_category_highlight_system,
    update_creative_visibility_system, update_pager_button_highlight_system,
};
pub use skin::{apply_creative_tab_icon_system, apply_creative_title_icon_system};
