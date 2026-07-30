//! 组装创造物品栏的分类、网格、快捷栏和界面交互。

mod catalog;
mod grid;
mod hotbar;
mod interaction;

pub use crate::client::ui::screens::setup::spawn_creative_inventory_system;
pub use catalog::{build_creative_categories_system, update_creative_filter_system};
pub use grid::{populate_creative_grid_system, populate_recent_panel_system};
pub use hotbar::{
    cleanup_creative_hotbar_system, creative_hotbar_visual_sync_system, init_creative_hotbar_system,
};
pub use interaction::{
    creative_close_button_system, sync_creative_search_placeholder_system,
    update_category_highlight_system, update_creative_visibility_system,
};
