//! 保存水面材质与水下滤镜的表现常量。

/// 水面流动速度（兼容旧版 UV 动画接口）。
pub const WATER_FLOW_SPEED: f32 = 0.12;

/// 水面流动的单 tile UV 周期（兼容旧版 UV 动画接口）。
pub const WATER_FLOW_TILE: f32 = 1.0;

/// 水面材质时间速度（无量纲）。
pub const WATER_SURFACE_TIME_SCALE: f32 = 1.0;

/// 水下视觉颜色 grading 的曝光偏移。
pub const UNDERWATER_COLOR_GRADE_EXPOSURE: f32 = -0.38;

/// 水下视觉颜色 grading 的饱和度。
pub const UNDERWATER_COLOR_GRADE_SATURATION: f32 = 0.82;

/// 水下深度过渡速率（每帧向目标逼近的比例）。
pub const UNDERWATER_DEPTH_RATE: f32 = 6.0;

/// 水下滤镜最大覆盖层透明度。
pub const UNDERWATER_OVERLAY_MAX_ALPHA: f32 = 0.20;

/// 水下雾效颜色。
pub const UNDERWATER_FOG_COLOR: [f32; 3] = [0.025, 0.17, 0.31];

/// 水下雾效能见距离范围（近远）。
pub const UNDERWATER_FOG_NEAR: f32 = 3.5;

/// 水下雾效终点。
pub const UNDERWATER_FOG_FAR: f32 = 34.0;

/// 水下曝光偏移（EV100 降低，保持水下压迫感但不压黑细节）。
pub const UNDERWATER_EXPOSURE_OFFSET: f32 = -1.65;

/// 水下体积雾环境色。
pub const UNDERWATER_VOLUMETRIC_COLOR: [f32; 3] = [0.015, 0.13, 0.29];
