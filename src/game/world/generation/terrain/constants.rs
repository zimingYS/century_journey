//! 保存会影响确定性地形生成结果的固定参数。
//!
//! 调整数值会改变同一种子的地形结果，因此必须同时评估并更新世界生成版本。

/// 海平面的世界方块高度。
pub(in crate::game::world) const SEA_LEVEL: i32 = 64;

/// 主地形噪声的世界坐标缩放。
pub(super) const GLOBAL_TERRAIN_SCALE: f64 = 0.005;

/// 小尺度地形细节噪声的世界坐标缩放。
pub(super) const GLOBAL_DETAIL_SCALE: f64 = 0.02;

/// 地形粗糙度噪声的世界坐标缩放。
pub(super) const GLOBAL_ROUGHNESS_SCALE: f64 = 0.01;
