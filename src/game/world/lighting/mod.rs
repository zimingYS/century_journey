//! 组织权威光照数据与方块光传播规则。
//!
//! 光级数组（`ChunkLight`）是会话期世界状态：固定步负责版本与提交，任务线程
//! 执行局部优先传播和低频全局校正，客户端网格通过快照消费；不进存档。

pub mod chunk_light;
mod local;
mod local_queue;
mod plugin;
pub mod rebuild;
mod resources;
mod systems;

pub use plugin::LightingPlugin;
pub use resources::WorldLighting;

#[cfg(test)]
#[path = "../../../../tests/unit/game/world/lighting/mod.rs"]
mod tests;
