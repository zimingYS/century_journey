//! 提供客户端 HUD、菜单、物品栏界面、导航和通用控件。

pub mod components;
pub mod hud;
pub mod interaction;
pub mod navigation;
mod plugin;
pub mod resources;
pub mod screens;
pub mod screenshot_check;
pub mod state;
pub mod theme;
pub mod widgets;

pub use plugin::UIPlugin;
