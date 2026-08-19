//! 体积云扩展材质及其 GPU 参数块。
//!
//! 云复用 `StandardMaterial` 的顶点结构（提供世界坐标与世界法线），在扩展
//! fragment shader 里做 raymarching：从摄像机位置沿视线在云层高度范围内步进，
//! 采样分形噪声并结合垂直剖面累积光学厚度，产生有侧面、有厚度的体积云。
//!
//! 本模块是原 raymarching 体积云的保留备份（当前未注册，见 `mod.rs`），作为
//! 未来切换回体积云的入口。dead_code 豁免声明在 `mod.rs` 的模块声明上。

use bevy::math::Vec4;
use bevy::pbr::{ExtendedMaterial, MaterialExtension, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;

/// 与 WGSL `CloudVolumeExtension` 逐字段对应的单一 uniform 参数块。
#[derive(Clone, Copy, Debug, Reflect, ShaderType)]
pub struct CloudVolumeUniform {
    /// 当前视觉时间（秒），驱动风漂移。
    pub time_seconds: f32,
    /// 云覆盖率 0~1，来自天气适配器；越高云越多。
    pub coverage: f32,
    /// 昼夜因子 0~1（0=白天，1=深夜）。
    pub night_factor: f32,
    /// 黄昏峰值强度 0~1。
    pub twilight_glow: f32,
    /// 云体下沿世界高度。
    pub cloud_min_y: f32,
    /// 云体上沿世界高度。
    pub cloud_max_y: f32,
    /// 风漂移速度（世界单位/秒），驱动噪声采样平移。
    pub wind_speed: f32,
    /// 噪声采样频率；越小云团越大块。
    pub noise_scale: f32,
    /// 云体素尺寸（世界单位），决定"方块云"的块大小。
    pub cell_size: f32,
    /// 内容定义的云密度阈值；值越高，云团越稀疏。
    pub density_threshold: f32,
    /// 高频噪声侵蚀强度，控制块状云边缘的碎裂程度。
    pub detail_strength: f32,
    /// 远景能见度，雾效或恶劣天气会降低最终不透明度。
    pub visibility: f32,
    /// 摄像机世界位置（xyz），作为光线步进起点。
    pub camera_position: Vec4,
    /// 太阳方向（xyz，归一化），用于云的自遮蔽光照。
    pub sun_direction: Vec4,
    /// 水平风向（xy，归一化），驱动云的整体漂移方向。
    pub wind_direction: Vec4,
    /// 白天色调（RGB）+ 不透明度（A）。
    pub tint_day: Vec4,
    /// 夜晚色调（RGB）。
    pub tint_night: Vec4,
    /// 黄昏叠加色调（RGB）。
    pub tint_sunset: Vec4,
}

impl Default for CloudVolumeUniform {
    fn default() -> Self {
        Self {
            time_seconds: 0.0,
            coverage: 0.1,
            night_factor: 0.0,
            twilight_glow: 0.0,
            cloud_min_y: 104.0,
            cloud_max_y: 112.0,
            wind_speed: 4.0,
            noise_scale: 0.004,
            cell_size: 8.0,
            density_threshold: 0.62,
            detail_strength: 0.32,
            visibility: 1.0,
            camera_position: Vec4::new(0.0, 0.0, 0.0, 0.0),
            sun_direction: Vec4::new(0.35, 0.82, 0.25, 0.0),
            wind_direction: Vec4::new(1.0, 0.0, 0.0, 0.0),
            tint_day: Vec4::new(1.0, 1.0, 1.0, 0.85),
            tint_night: Vec4::new(0.22, 0.25, 0.36, 0.85),
            tint_sunset: Vec4::new(1.0, 0.72, 0.52, 0.85),
        }
    }
}

/// 体积云扩展材质；所有参数必须共用绑定 100，与 WGSL 参数块保持一致。
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct CloudVolumeExtension {
    /// GPU 侧连续读取的云参数块。
    #[uniform(100)]
    pub uniform: CloudVolumeUniform,
}

impl MaterialExtension for CloudVolumeExtension {
    fn fragment_shader() -> ShaderRef {
        "shaders/cloud_volume.wgsl".into()
    }

    fn enable_prepass() -> bool {
        // 云是半透明表现层，不写入深度预通道。
        false
    }

    fn enable_shadows() -> bool {
        // 云不参与阴影计算。
        false
    }
}

/// 体积云使用的完整材质类型。
pub type CloudVolumeMaterial = ExtendedMaterial<StandardMaterial, CloudVolumeExtension>;

#[cfg(test)]
#[path = "../../../../tests/unit/client/sky/cloud/material.rs"]
mod tests;
