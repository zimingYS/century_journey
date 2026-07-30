//! 组装生存物品栏的布局、预览、交互命令和视觉同步系统。

mod actions;
mod layout;
mod preview;
mod sync;

pub use actions::backpack_management_button_system;
pub use layout::spawn_survival_inventory_system;
pub use sync::{
    init_survival_hotbar_system, populate_survival_grid_system, survival_grid_visual_sync_system,
    survival_hotbar_visual_sync_system, survival_stats_visual_sync_system,
    sync_accessory_slot_count_system, update_survival_visibility_system,
};
