//! 引擎任务门面。
//!
//! CPU 与 IO 任务分别复用 Bevy 的 AsyncComputeTaskPool 和 IoTaskPool。
//! 本模块只暴露项目实际使用的句柄、管理器、运行时插件与统计数据，
//! 不维护另一套线程池或调度器。

pub(crate) mod diagnostics;
pub(crate) mod job;
pub mod manager;
pub mod plugin;
pub(crate) mod runtime;

pub use job::{TaskHandle, TaskId, TaskResult};
pub use manager::TaskManager;
pub use plugin::TaskPlugin;
