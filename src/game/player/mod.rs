pub mod components;
pub mod systems;
pub mod events;
pub mod model;
pub mod plugin;

/// 向后兼容：PlayerPlugin 等同 GamePlayerPlugin。
pub use plugin::GamePlayerPlugin as PlayerPlugin;
