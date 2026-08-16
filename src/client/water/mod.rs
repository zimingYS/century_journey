//! 组织水面动态材质与水下视觉滤镜。
//!
//! 本模块是纯表现层：水面材质在 GPU 中计算波法线、深度渐变和岸线泡沫，
//! 水下滤镜检测玩家头部浸没并投影到相机色调、雾效和曝光。
//! 不参与 FixedUpdate，不改变任何权威游戏规则。

mod components;
mod constants;
mod material;
mod plugin;
mod systems;

pub use plugin::WaterPlugin;

#[doc(hidden)]
pub use components::UnderwaterOverlay;
#[doc(hidden)]
pub use constants::{WATER_FLOW_SPEED, WATER_FLOW_TILE};
#[doc(hidden)]
pub use material::{WaterMaterial, WaterMaterialExtension};
#[doc(hidden)]
pub use systems::{
    compute_underwater_alpha, underwater_depth_step, water_depth_factor, water_flow_offset,
};
