//! 聚合玩家身份、控制、移动、物理、交互、战斗、生存和生命周期领域。

pub mod combat;
pub mod control;
pub mod identity;
pub mod interaction;
pub mod lifecycle;
pub mod movement;
pub mod physics;
pub mod plugin;
pub mod survival;

/// 向后兼容：PlayerPlugin 等同 GamePlayerPlugin。
pub use plugin::GamePlayerPlugin as PlayerPlugin;
