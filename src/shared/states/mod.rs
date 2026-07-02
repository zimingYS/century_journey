//! # States
//!
//! 公共状态。
//!
//! 定义应用及游戏运行过程中共享的 State。

pub mod app_state;
pub mod input_blocked;

pub use app_state::AppState;
pub use input_blocked::InputBlocked;
