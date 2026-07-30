//! 定义树木逻辑实例，并维护按根区块归属的确定性实例索引。

mod lifecycle;
mod model;
mod store;

pub(super) use lifecycle::track_tree_root_changes_system;
pub(in crate::game) use model::{TreeGrowthStage, TreeInstance};
pub(in crate::game::world) use store::TreeInstanceStore;
