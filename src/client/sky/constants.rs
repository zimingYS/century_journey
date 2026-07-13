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

/// 月光最大照度（满月），与 light_consts::lux::FULL_MOON_NIGHT 一致
pub const MAX_MOON_ILLUMINANCE: f32 = 0.27;

/// 月光最小照度（地平线以下时）
pub const MIN_MOON_ILLUMINANCE: f32 = 0.0;

/// 夜间VolumetricFog环境光强度
pub const NIGHT_FOG_AMBIENT: f32 = 0.02;

/// 白天VolumetricFog环境光强度
pub const DAY_FOG_AMBIENT: f32 = 0.1;

/// 日出/日落VolumetricFog环境光强度
pub const TWILIGHT_FOG_AMBIENT: f32 = 0.06;
