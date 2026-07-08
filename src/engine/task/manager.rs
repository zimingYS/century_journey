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
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        self.spawn_cpu(task)
    }

    pub fn spawn_cpu(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        let id = TaskId::new();
        self.spawn_on_async_compute_pool(task);
        TaskHandle::new(id)
    }

    pub fn spawn_io(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        let id = TaskId::new();
        self.spawn_on_io_pool(task);
        TaskHandle::new(id)
    }

    pub fn pending_count(&self) -> usize {
        self.counters.active.load(Ordering::Relaxed)
    }

    pub fn completed_count(&self) -> u64 {
        self.counters.completed.load(Ordering::Relaxed)
    }

    pub fn failed_count(&self) -> u64 {
        self.counters.failed.load(Ordering::Relaxed)
    }

    pub fn worker_count(&self) -> usize {
        AsyncComputeTaskPool::get().thread_num()
    }

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
