//! 汇总 App 与其他顶层模块共享的稳定状态契约。

pub mod app_state;
pub mod input_context;

pub use app_state::AppState;
pub use input_context::{InputContext, InputContextState};
