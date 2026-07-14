/// 日月纹理尺寸
pub const CELESTIAL_MESH_SIZE: f32 = 64.0;

/// 太阳/月亮到原点的距离（用于billboard定位）
pub const CELESTIAL_DISTANCE: f32 = 500.0;

/// 太阳纹理尺寸（像素）
pub const SUN_TEXTURE_SIZE: u32 = 128;

/// 月亮纹理尺寸（像素）
pub const MOON_TEXTURE_SIZE: u32 = 128;

/// 星星纹理尺寸（像素）
pub const STAR_TEXTURE_SIZE: u32 = 8;

/// 星星数量
pub const STAR_COUNT: usize = 800;

/// 星星散布球面半径
pub const STAR_SPHERE_RADIUS: f32 = 480.0;

/// 星星Mesh面片大小
pub const STAR_QUAD_SIZE: f32 = 2.0;

/// 月光最大照度。为保证体素地形可读性，使用高于物理满月的玩法标定值。
pub const MAX_MOON_ILLUMINANCE: f32 = 2.5;

/// 月光最小照度（地平线以下时）
pub const MIN_MOON_ILLUMINANCE: f32 = 0.05;

/// 深夜相机曝光值；EV100 越低，画面越亮。
pub const NIGHT_EXPOSURE_EV100: f32 = 5.5;

/// 白天和深夜的全局环境光亮度。
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 80.0;
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 22.0;

/// 夜间VolumetricFog环境光强度
pub const NIGHT_FOG_AMBIENT: f32 = 0.07;

/// 白天VolumetricFog环境光强度
pub const DAY_FOG_AMBIENT: f32 = 0.1;

/// 日出/日落VolumetricFog环境光强度
pub const TWILIGHT_FOG_AMBIENT: f32 = 0.06;
