//! 组织生命、护甲、饥饿等生存状态条的左右布局。

pub mod left_bars;
pub mod right_bars;

mod components;
mod layout;
mod resources;
mod systems;

pub use components::{BarsHud, LeftBarsHud, RightBarsHud};
pub use layout::{
    HUD_STATUS_ICON_GAP, HUD_STATUS_ICON_SIZE, hud_hotbar_outer_width, shown_status_units,
    status_icon_count, status_icon_node, status_icon_segment,
};
pub use resources::{HudStatusIconAssets, StatusIconSegment};
pub use systems::{load_hud_status_icon_assets_system, spawn_bars_hud_system};
