//! 世界方块变更管道：命令缓冲 + 单一应用点 + 变更来源稀疏层。
//!
//! 纯数据类型（ChangeSource/VoxelChange/Buffer）在 `shared::voxel_change`，
//! 此处 re-export 保持既有引用路径不变；apply 实现与来源记录留在 Game 层。

pub mod apply;
pub mod provenance;
