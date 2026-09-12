//! cj-world —— 世界状态与存储
//!
//! 规则：
//! - 不依赖 runtime / render / game
//! - 玩家扰动（delta）与自然积分量（integral）必须分开（§6.6.1 / 评审 #23）

pub mod chunk;
pub mod delta;
pub mod integral;
pub mod seed;
