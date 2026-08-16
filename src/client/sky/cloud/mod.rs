//! 组织多层动态云的实体创建与每帧表现更新。
//!
//! 云是纯表现层功能：读取 Content 编译的云定义与客户端时间快照，在渲染帧
//! 驱动云层漂移、昼夜染色与近景云片朝向。不进入 FixedUpdate，不参与权威模拟。

mod components;
mod constants;
mod plugin;
mod systems;
mod texture;
mod weather_adapter;

pub use plugin::CloudPlugin;

/// 供单元测试与天气适配器使用的类型入口。
#[doc(hidden)]
pub use components::{CloudLayer, CloudPatch, CloudWeatherState};
#[doc(hidden)]
pub use constants::CLOUD_TEXTURE_SIZE;
#[doc(hidden)]
pub use texture::generate_cloud_texture;
