//! 组织游戏内常驻抬头显示及其布局根节点。

pub mod bottom;
pub mod center;
mod components;
pub mod left;
pub mod left_bottom;
pub mod left_top;
pub mod plugin;
pub mod right;
pub mod right_bottom;
pub mod right_top;
mod systems;
pub mod top;

pub use components::HudRoot;
pub use systems::spawn_hud_root_system;
