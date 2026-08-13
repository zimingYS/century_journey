//! 区块体素光专用 PBR 扩展材质。
//!
//! 网格顶点色保存天空光与方块光合成后的乘法光色，第二组 UV 保存独立的
//! 4bit RGB 方块光。扩展着色器把后者作为稳定的间接照明加入 PBR，避免
//! Bevy 实体点光离开近景预算后，远处火把光突然消失。

use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// 方块光作为屏幕空间间接照明的标定强度。
const DEFAULT_BLOCK_INDIRECT_STRENGTH: f32 = 0.85;
/// 无光环境保留的中性贴图可读度；不参与世界光传播和颜色混合。
const DEFAULT_DARK_SURFACE_STRENGTH: f32 = 0.08;

/// 与 WGSL 参数块逐字段对应的区块体素光参数。
#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
struct VoxelMaterialUniform {
    block_indirect_strength: f32,
    dark_surface_strength: f32,
    _padding_y: f32,
    _padding_z: f32,
}

impl Default for VoxelMaterialUniform {
    fn default() -> Self {
        Self {
            block_indirect_strength: DEFAULT_BLOCK_INDIRECT_STRENGTH,
            dark_surface_strength: DEFAULT_DARK_SURFACE_STRENGTH,
            _padding_y: 0.0,
            _padding_z: 0.0,
        }
    }
}

/// 为世界区块补充远景方块光的材质扩展。
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub(crate) struct VoxelMaterialExtension {
    /// GPU 侧连续读取的体素光参数块。
    #[uniform(100)]
    uniform: VoxelMaterialUniform,
}

impl MaterialExtension for VoxelMaterialExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/voxel_lighting.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/voxel_lighting.wgsl".into()
    }
}

/// 世界区块使用的完整体素光材质类型。
pub(crate) type VoxelMaterial = ExtendedMaterial<StandardMaterial, VoxelMaterialExtension>;

#[cfg(test)]
#[path = "../../../../tests/unit/client/renderer/lighting/material.rs"]
mod tests;
