//! 合成网格领域模型及配方匹配入口。
//!
//! 本模块对外保持原有 `crafting::grid` 公共路径，内部将容器状态与匹配算法分离，
//! 避免网格数据、ECS 组件和递归匹配规则继续堆叠在同一文件中。

mod matching;
mod model;

pub use model::{ActiveCrafting, CraftingGrid, PlayerCrafting, WorkbenchCrafting};

#[cfg(test)]
#[path = "../../../../tests/unit/game/crafting/grid/mod.rs"]
mod tests;
