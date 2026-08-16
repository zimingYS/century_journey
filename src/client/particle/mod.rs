//! 轻量级客户端粒子反馈。
//!
//! 粒子由已经发生的方块事件和动画标记生成，不承担命中或方块变更判定。

mod plugin;
mod systems;
mod types;

pub use plugin::ClientParticlePlugin;

#[cfg(test)]
#[path = "../../../tests/unit/client/particle/mod.rs"]
mod tests;
