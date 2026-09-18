//! cj-render —— 渲染层
//!
//! 规则：只消费渲染数据，不知道模拟算法（不依赖 sim）。

pub mod camera;
pub mod input;
pub mod meshing;
pub mod plugin;
pub mod resources;
pub mod terrain;

pub use plugin::RenderPlugin;
