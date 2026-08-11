//! 保存天空表现专用的尺寸、颜色和光照常量。

/// 日月纹理尺寸。
pub const CELESTIAL_MESH_SIZE: f32 = 38.0;

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

/// 阳光强度有意低于真实物理值，使体素反照率和玩家背光面能够同时保持清晰。
pub const DAY_SUN_ILLUMINANCE: f32 = 72_000.0;

/// 月光最大照度。为保证体素地形可读性，使用高于物理满月的玩法标定值。
pub const MAX_MOON_ILLUMINANCE: f32 = 2.5;

/// 月光最小照度（地平线以下时）
pub const MIN_MOON_ILLUMINANCE: f32 = 0.05;

/// 深夜相机曝光值；EV100 越低，画面越亮。
pub const NIGHT_EXPOSURE_EV100: f32 = 5.5;

/// 白天和深夜的全局环境光亮度。
///
/// 顶点光色采用乘法衰减系数（露天=1.0、洞穴无光≈0），洞穴纯黑由
/// 顶点色自动保证，不受本值影响；本值只权衡露天背光面可见性与
/// 洞穴火把区域的氛围亮度。
pub const DAY_AMBIENT_BRIGHTNESS: f32 = 45.0;
/// 夜间环境光的最低亮度，避免无光源区域完全不可见。
pub const NIGHT_AMBIENT_BRIGHTNESS: f32 = 6.0;

/// 夜间VolumetricFog环境光强度
pub const NIGHT_FOG_AMBIENT: f32 = 0.09;

/// 白天VolumetricFog环境光强度
pub const DAY_FOG_AMBIENT: f32 = 0.16;

/// 日出/日落VolumetricFog环境光强度
pub const TWILIGHT_FOG_AMBIENT: f32 = 0.12;
