//! 管理与具体玩法无关的异步任务提交和句柄。

use crate::engine::task::job::{TaskHandle, TaskId, TaskResult};
use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, IoTaskPool};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Default)]
struct TaskCounters {
    active: AtomicUsize,
    completed: AtomicU64,
    failed: AtomicU64,
}

#[derive(Resource, Clone)]
/// 提交通用 CPU/IO 任务并维护运行统计的管理器。
pub struct TaskManager {
    counters: Arc<TaskCounters>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            counters: Arc::new(TaskCounters::default()),
        }
    }
}

impl TaskManager {
    /// 使用给定参数创建新实例。
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// 向默认执行器提交一个通用异步任务。
    pub fn spawn(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        self.spawn_cpu(task)
    }

    /// 向 CPU 执行器提交计算密集任务。
    pub fn spawn_cpu(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        let id = TaskId::new();
        self.spawn_on_async_compute_pool(task);
        TaskHandle::new(id)
    }

    /// 向 IO 执行器提交阻塞型任务。
    pub fn spawn_io(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        let id = TaskId::new();
        self.spawn_on_io_pool(task);
        TaskHandle::new(id)
    }

    /// 返回仍未完成的任务数量。
    pub fn pending_count(&self) -> usize {
        self.counters.active.load(Ordering::Relaxed)
    }

    /// 返回运行时累计完成的任务数量。
    pub fn completed_count(&self) -> u64 {
        self.counters.completed.load(Ordering::Relaxed)
    }

    /// 返回运行时累计失败的任务数量。
    pub fn failed_count(&self) -> u64 {
        self.counters.failed.load(Ordering::Relaxed)
    }

    /// 返回任务运行时配置的工作线程数量。
    pub fn worker_count(&self) -> usize {
        AsyncComputeTaskPool::get().thread_num()
    }

    /// 返回当前正在执行的任务数量。
    pub fn running_count(&self) -> usize {
        self.pending_count()
    }

    fn spawn_on_async_compute_pool(&self, task: impl FnOnce() -> TaskResult + Send + 'static) {
        let counters = self.counters.clone();
        counters.active.fetch_add(1, Ordering::Relaxed);
        AsyncComputeTaskPool::get()
            .spawn(async move {
                record_task_result(&counters, task());
            })
            .detach();
    }

    fn spawn_on_io_pool(&self, task: impl FnOnce() -> TaskResult + Send + 'static) {
        let counters = self.counters.clone();
        counters.active.fetch_add(1, Ordering::Relaxed);
        IoTaskPool::get()
            .spawn(async move {
                record_task_result(&counters, task());
            })
            .detach();
    }
}

fn record_task_result(counters: &TaskCounters, result: TaskResult) {
    if matches!(result, TaskResult::Failed(_)) {
        counters.failed.fetch_add(1, Ordering::Relaxed);
    }
    counters.completed.fetch_add(1, Ordering::Relaxed);
    counters.active.fetch_sub(1, Ordering::Relaxed);
}
