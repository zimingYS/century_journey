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

/// 日出开始时间（小时）
pub const SUNRISE_START: f32 = 5.0;

/// 日出结束时间（小时）
pub const SUNRISE_END: f32 = 7.0;

/// 日落开始时间（小时）
pub const SUNSET_START: f32 = 17.0;

/// 日落结束时间（小时）
pub const SUNSET_END: f32 = 19.0;

/// 夜间环境光颜色（深蓝）
pub const NIGHT_AMBIENT_COLOR: [f32; 3] = [0.04, 0.04, 0.12];

/// 白天环境光颜色（暖白）
pub const DAY_AMBIENT_COLOR: [f32; 3] = [0.6, 0.55, 0.45];

/// 日出/日落环境光颜色（暖橙）
pub const TWILIGHT_AMBIENT_COLOR: [f32; 3] = [0.7, 0.35, 0.15];

/// 夜间VolumetricFog环境光强度
pub const NIGHT_FOG_AMBIENT: f32 = 0.02;

/// 白天VolumetricFog环境光强度
pub const DAY_FOG_AMBIENT: f32 = 0.1;

/// 日出/日落VolumetricFog环境光强度
pub const TWILIGHT_FOG_AMBIENT: f32 = 0.06;

/// 星星白天透明度（完全隐藏）
pub const STAR_DAY_ALPHA: f32 = 0.0;

/// 星星夜晚透明度
pub const STAR_NIGHT_ALPHA: f32 = 1.0;
