//! # Engine Task System V3
//!
//! 统一任务调度系统 + 并行计算运行时。
//!
//! ## API
//!
//! | 方法 | 说明 |
//! |------|------|
//! | `TaskManager::parallel_for(data, f)` | 并行 For |
//! | `TaskManager::parallel_map(data, f)` | 并行 Map |
//! | `TaskManager::parallel_reduce(data, map, reduce, id)` | 并行 Reduce |
//! | `TaskManager::fork_join(tasks)` | Fork-Join |
//! | `TaskManager::dispatch_batch(data, strategy, f)` | Batch 分发 |

pub mod cancellation;
pub mod dependency;
pub mod diagnostics;
pub mod executor;
pub mod group;
pub mod job;
pub mod manager;
pub mod plugin;
pub mod runtime;
pub mod scheduler;
pub mod util;
pub mod worker;

pub use cancellation::{CancellationManager, CancellationSource, CancellationToken};
pub use dependency::{DependencyEdge, DependencyGraph, DependencyNode, DependencyResolver};
pub use diagnostics::{RuntimeStatistics, TaskReport};
pub use executor::{
    BatchDispatcher, BatchResult, ForkJoin, ParallelExecutor, ParallelFor, Partition,
    PartitionStrategy, Reducer, TaskBarrier,
};
pub use group::{TaskGroup, TaskGroupHandle, TaskScope};
pub use job::{TaskDependency, TaskHandle, TaskId, TaskJob, TaskResult, TaskState};
pub use manager::TaskManager;
pub use plugin::TaskPlugin;
pub use runtime::{
    RuntimeContext, RuntimeService, RuntimeServices, RuntimeState, TaskRuntimePlugin,
};
pub use scheduler::{
    DispatchDecision, DispatchPipeline, FrameBudget, TaskPriority, TaskQueue, TaskScheduler,
};
pub use worker::{
    LocalQueue, WorkStealer, Worker, WorkerGroup, WorkerKind, WorkerPool, WorkerStatistics,
};
