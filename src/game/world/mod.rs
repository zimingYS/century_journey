//! 提供 Game 层的权威世界规则及其领域子模块。

pub mod block_ops;
pub mod chunk;
pub mod entity;
pub mod generation;
pub mod interaction;
mod plugin;
pub mod state;
pub mod streaming;
mod structure;
pub mod time;
mod vegetation;

pub use plugin::GameWorldPlugin;
pub(in crate::game) use vegetation::{TreeGrowthStage, TreeInstance};
