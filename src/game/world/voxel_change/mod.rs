//! 世界方块变更管道：命令缓冲 + 单一应用点。
//!
//! 纯数据类型（ChangeSource/VoxelChange/Buffer）在 `shared::voxel_change`，
//! 此处 re-export 保持既有引用路径不变；apply 实现留在 Game 层。

pub mod apply;
