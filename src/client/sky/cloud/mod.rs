//! 组织多层动态云的实体创建与每帧表现更新。
//!
//! 当前实现：voxel 块状云（ARTShade 风格几何云）。raymarching 体积云代码
//! 仍保留在子模块（`material`、`systems`、`texture`）中以备未来切换，但
//! 不在 plugin 中注册，因此不会被运行时使用。

mod components;
mod constants;
mod generation;
mod material;
mod plugin;
mod systems;
mod texture;
mod voxel;
mod weather_adapter;

pub use plugin::CloudPlugin;

/// 供单元测试与天气适配器使用的类型入口。
#[doc(hidden)]
pub use components::{CloudLayer, CloudPatch, CloudWeatherState};
#[doc(hidden)]
pub use constants::CLOUD_TEXTURE_SIZE;
#[doc(hidden)]
pub use texture::generate_cloud_texture;
