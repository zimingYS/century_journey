use crate::engine::task::scheduler::TaskScheduler;
use crate::engine::task::worker::worker::Worker;
use std::sync::{Arc, Mutex};

/// Worker 线程池
///
/// 负责创建和管理 Worker 的生命周期。
/// 不管理 Queue（Queue 由 Scheduler 管理）。
pub struct WorkerPool {
    workers: Vec<Worker>,
    scheduler: Arc<Mutex<TaskScheduler>>,
}

impl WorkerPool {
    /// 创建 WorkerPool 并启动指定数量的 Worker
    pub fn new(scheduler: Arc<Mutex<TaskScheduler>>, count: usize) -> Self {
        let mut pool = Self {
            workers: Vec::with_capacity(count),
            scheduler,
        };

        for _ in 0..count {
            pool.workers.push(Worker::spawn(pool.scheduler.clone()));
        }

        pool
    }

    /// 以 CPU 核心数创建
    pub fn with_thread_count(scheduler: Arc<Mutex<TaskScheduler>>) -> Self {
        let count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);
        Self::new(scheduler, count)
    }

    /// 当前 Worker 数量
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }

    /// 停止所有 Worker
    pub fn stop_all(&mut self) {
        for worker in &mut self.workers {
            worker.stop();
        }
        self.workers.clear();
    }
}

impl Drop for WorkerPool {
    fn drop(&mut self) {
        self.stop_all();
    }
}
