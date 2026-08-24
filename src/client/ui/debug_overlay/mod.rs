//! F3 调试浮层：开发期叠加在世界画面左上角的实时状态文本。

pub mod components;
mod info;
pub mod plugin;
mod systems;

pub use plugin::DebugOverlayPlugin;
