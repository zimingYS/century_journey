//! 组织跨区块结构放置与延迟写入，保证生成顺序不影响结果。

pub mod pending_writes;
pub mod placement;
