pub mod action;
pub mod combat;
pub mod command;
pub mod events;
pub mod identity;
pub mod interaction;
pub mod lifecycle;
pub mod movement;
pub mod physics;
pub mod plugin;
pub mod spawn;
pub mod survival;

/// 向后兼容：PlayerPlugin 等同 GamePlayerPlugin。
pub use plugin::GamePlayerPlugin as PlayerPlugin;
