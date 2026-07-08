//! # Engine Task Facade
//!
//! Runtime dispatch uses Bevy's `AsyncComputeTaskPool` for CPU work and
//! `IoTaskPool` for IO work. The legacy scheduler, dependency, cancellation,
//! executor, group, and worker implementations have been removed from the
//! active compile path so the API matches the project's current behavior.
//!
//! # 引擎任务门面
//!
//! 运行时派发使用 Bevy 的 `AsyncComputeTaskPool` 执行 CPU 任务，并使用
//! `IoTaskPool` 执行 IO 任务。旧的 scheduler、dependency、cancellation、
//! executor、group 和 worker 实现已从活动编译路径移除，让 API 与项目当前行为一致。

pub(crate) mod diagnostics;
pub(crate) mod job;
pub mod manager;
pub mod plugin;
pub(crate) mod runtime;

pub use job::{TaskHandle, TaskId, TaskResult};
pub use manager::TaskManager;
pub use plugin::TaskPlugin;
