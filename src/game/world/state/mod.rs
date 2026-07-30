//! 组织权威世界数据、区块运行时状态和无渲染运行入口。

mod authoritative;
mod chunk_runtime;
mod headless;

pub(in crate::game) use authoritative::WorldChunkSnapshot;
pub use authoritative::WorldState;
pub use chunk_runtime::ChunkRuntime;
pub use headless::HeadlessWorldPlugin;
