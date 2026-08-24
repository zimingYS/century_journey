//! 采集客户端输入，并把界面意图与玩家动作转换为权威层命令。

mod actions;
mod context;
mod cursor;
mod interface;
mod plugin;
mod pointer;
mod rebind;

pub use actions::ClientActionState;
pub use context::{InputBlocked, InputSet};
pub use interface::InterfaceCommand;
pub use plugin::ClientInputPlugin;
pub use pointer::{UiInteractionLifecycleEvent, UiInteractionPhase};
pub use rebind::RebindCapture;

pub mod console;
#[cfg(test)]
#[path = "../../../tests/unit/client/input/mod.rs"]
mod tests;
