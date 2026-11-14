//! cj-game —— 玩法层
//!
//! 规则：不重新实现底层模拟算法（不依赖 sim / world 的算法）。

pub mod input;
pub mod interaction;
pub mod player;
pub mod plugin;
pub mod sets;
pub mod tools;

pub use plugin::GamePlugin;
