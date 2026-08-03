//! 定义云场渲染实体和可由天气模拟驱动的表现状态。

use crate::content::cloud::definition::CloudLayerDefinition;
use bevy::prelude::*;

/// 云场表现状态的轻量输入接口。
///
/// 当前默认值提供稳定的晴天云层；未来 Game 层只需在跨层适配器中更新云量和
/// 风速倍率，渲染实体、纹理和调度顺序无需改变。
#[derive(Resource, Debug, Clone, Copy)]
pub struct CloudWeatherState {
    /// 云量权重，0 表示完全淡出，1 表示使用定义中的完整不透明度。
    pub coverage: f32,
    /// 风速倍率，天气模拟可用它表达阵风或静风。
    pub wind_multiplier: f32,
    /// 远景能见度权重，用于雾或恶劣天气统一降低云层对比度。
    pub visibility: f32,
}

impl Default for CloudWeatherState {
    fn default() -> Self {
        Self {
            coverage: 1.0,
            wind_multiplier: 0.1,
            visibility: 1.0,
        }
    }
}

impl CloudWeatherState {
    /// 返回经过有限范围约束的表现参数，防止外部天气输入破坏材质颜色。
    pub fn normalized(self) -> Self {
        let finite_or = |value: f32, fallback: f32| {
            if value.is_finite() { value } else { fallback }
        };
        Self {
            coverage: finite_or(self.coverage, 1.0).clamp(0.0, 1.0),
            wind_multiplier: finite_or(self.wind_multiplier, 1.0).clamp(0.0, 4.0),
            visibility: finite_or(self.visibility, 1.0).clamp(0.0, 1.0),
        }
    }
}

/// 云层实体组件，保存静态层定义与世界连续 UV 相位。
#[derive(Component)]
pub struct CloudLayer {
    /// 云层定义参数。
    pub definition: CloudLayerDefinition,
    /// 当前纹理相位；使用 Repeat 采样后始终保持在有限范围。
    pub uv_offset: Vec2,
    /// 上一帧相机水平位置，用于补偿相机移动并保持云场世界连续。
    pub last_camera_position: Vec2,
}

/// 近景 billboard 云片组件。
#[derive(Component)]
pub struct CloudPatch {
    /// 云片世界尺寸（边长）。
    pub scale: f32,
    /// 云片环绕相机的重生半径。
    pub radius: f32,
}
