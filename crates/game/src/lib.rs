//! cj-game —— 玩法层
//!
//! 规则：不重新实现底层模拟算法（不依赖 sim / world 的算法）。

pub mod plugin;

pub use plugin::GamePlugin;
