//! 水面专用材质及其 GPU 参数。
//!
//! 水面不能继续复用普通半透明方块材质：它需要读取深度预通道，
//! 才能在岸线计算真实水深，并在同一张网格上生成波法线、高光和泡沫。

use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// 与 WGSL `WaterMaterialExtension` 逐字段对应的单一 uniform 参数块。
#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct WaterMaterialUniform {
    /// 当前视觉时间（秒），只用于客户端着色动画。
    pub time_seconds: f32,
    /// 波法线的空间频率。
    pub wave_scale: f32,
    /// 岸线泡沫亮度。
    pub foam_strength: f32,
    /// 深度渐变的最大采样距离（世界单位）。
    pub depth_fade: f32,
}

impl Default for WaterMaterialUniform {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            wave_scale: 0.9,
            foam_strength: 1.0,
            depth_fade: 6.0,
        }
    }
}

/// 水面扩展材质；所有参数必须共用绑定 100，与 WGSL 参数块保持一致。
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct WaterMaterialExtension {
    /// GPU 侧连续读取的水面参数块。
    #[uniform(100)]
    pub uniform: WaterMaterialUniform,
}

impl MaterialExtension for WaterMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/water_surface.wgsl".into()
    }

    fn enable_prepass() -> bool {
        // 水面只读取不透明场景深度，不能把自身写入深度预通道。
        false
    }

    fn enable_shadows() -> bool {
        false
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/water_surface.wgsl".into()
    }
}

/// 方块水网格使用的完整材质类型。
pub type WaterMaterial = ExtendedMaterial<StandardMaterial, WaterMaterialExtension>;

#[cfg(test)]
#[path = "../../../tests/unit/client/water/material.rs"]
mod tests;
