pub mod components;
pub mod events;
pub mod model;
pub mod plugin;
pub mod systems;

/// 向后兼容：PlayerPlugin 等同 GamePlayerPlugin。
pub use plugin::GamePlayerPlugin as PlayerPlugin;
