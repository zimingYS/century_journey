//! 实现已加载世界中的稀疏植被索引和权威树苗生长规则。

mod growth;
mod instance;
mod plugin;
mod runtime;
mod transition;

pub(in crate::game::world) use instance::TreeInstanceStore;
pub(in crate::game) use instance::{TreeGrowthStage, TreeInstance};
pub(in crate::game::world) use plugin::VegetationPlugin;
