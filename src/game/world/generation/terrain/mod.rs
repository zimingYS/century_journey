//! 组织基础地形、气候场、噪声源和生成上下文。

pub mod climate;
mod constants;
pub mod context;
pub mod generator;
pub mod noise;

pub(in crate::game::world) use constants::SEA_LEVEL;
