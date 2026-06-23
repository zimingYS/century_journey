use crate::engine::task::executor::batch::BatchResult;
use crate::engine::task::executor::executor::ParallelExecutor;
use crate::engine::task::executor::partition::PartitionStrategy;
use crate::engine::task::job::{TaskHandle, TaskJob, TaskResult};
use crate::engine::task::runtime::context::RuntimeContext;
use crate::engine::task::scheduler::{TaskPriority, TaskScheduler};
use crate::engine::task::worker::kind::WorkerKind;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Task管理器
#[derive(Resource)]
pub struct TaskManager {
    scheduler: Arc<Mutex<TaskScheduler>>,
    executor: ParallelExecutor,
}

impl TaskManager {
    pub(crate) fn new(scheduler: Arc<Mutex<TaskScheduler>>) -> Self {
        Self {
            scheduler,
            executor: ParallelExecutor::default(),
        }
    }

    /// 提交CPU任务
    pub fn spawn(&self, task: impl FnOnce() -> TaskResult + Send + 'static) -> TaskHandle {
        self.spawn_with_priority(TaskPriority::Normal, WorkerKind::Cpu, task)
    }

    /// 提交CPU任务（指定优先级）
    pub fn spawn_cpu(
        &self,
        priority: TaskPriority,
        task: impl FnOnce() -> TaskResult + Send + 'static,
    ) -> TaskHandle {
        self.spawn_with_priority(priority, WorkerKind::Cpu, task)
    }

    /// 提交IO任务
    pub fn spawn_io(
        &self,
        priority: TaskPriority,
        task: impl FnOnce() -> TaskResult + Send + 'static,
    ) -> TaskHandle {
        self.spawn_with_priority(priority, WorkerKind::Io, task)
    }

    /// 内部提交
    fn spawn_with_priority(
        &self,
        priority: TaskPriority,
        kind: WorkerKind,
        task: impl FnOnce() -> TaskResult + Send + 'static,
    ) -> TaskHandle {
        let job = TaskJob::new(priority, task).with_worker_kind(kind);
        let handle = TaskHandle::new(job.id);
        let mut sched = self.scheduler.lock().unwrap();
        sched.submit(job);
        handle
    }

    /// 等待所有依赖完成
    pub fn when_all(
        &self,
        handles: &[TaskHandle],
        task: impl FnOnce() -> TaskResult + Send + 'static,
    ) -> TaskHandle {
        let mut job = TaskJob::new(TaskPriority::Normal, task);
        for h in handles {
            job.dependencies.add(h.id());
        }
        let handle = TaskHandle::new(job.id);
        let mut sched = self.scheduler.lock().unwrap();
        sched.submit(job);
        handle
    }

    /// 取消指定任务
    pub fn cancel(&self, handle: &TaskHandle, ctx: &RuntimeContext) {
        ctx.cancellation.cancel(handle.id());
    }

    /// 收集已完成任务的结果
    pub fn collect_results(&self) -> Vec<TaskJob> {
        self.scheduler.lock().unwrap().collect_results()
    }

    /// 待处理任务数
    pub fn pending_count(&self) -> usize {
        self.scheduler.lock().unwrap().pending_count()
    }

    /// 已完成总数
    pub fn completed_count(&self) -> u64 {
        self.scheduler.lock().unwrap().completed_count()
    }

    /// Worker 数
    pub fn worker_count(&self) -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
    }

    pub fn running_count(&self) -> usize {
        self.worker_count()
    }

    pub fn scheduler(&self) -> Arc<Mutex<TaskScheduler>> {
        self.scheduler.clone()
    }

    /// 并行迭代
    pub fn parallel_for<T, F>(&self, data: Vec<T>, f: F)
    where
        T: Clone + Send + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        self.executor.parallel_for(data, f);
    }

    /// 并行映射
    pub fn parallel_map<T, R, F>(&self, data: Vec<T>, f: F) -> Vec<R>
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(T) -> R + Send + Sync + 'static,
    {
        self.executor.parallel_map(data, f)
    }

    /// 并行归约
    pub fn parallel_reduce<T, R, F, G>(&self, data: Vec<T>, map: F, reduce: G, identity: R) -> R
    where
        T: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> R + Send + Sync + 'static,
        G: Fn(R, R) -> R + Send + Sync + 'static,
    {
        self.executor.parallel_reduce(data, map, reduce, identity)
    }

    /// 分叉合并
    pub fn fork_join<F>(&self, tasks: Vec<F>) -> Vec<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.executor.fork_join(tasks)
    }

    /// 批量派发
    pub fn dispatch_batch<T, F>(
        &self,
        data: Vec<T>,
        strategy: PartitionStrategy,
        f: F,
    ) -> Vec<BatchResult>
    where
        T: Clone + Send + 'static,
        F: Fn(usize, Vec<T>) -> BatchResult + Send + Sync + 'static,
    {
        self.executor.dispatch_batch(data, strategy, f)
    }
}
