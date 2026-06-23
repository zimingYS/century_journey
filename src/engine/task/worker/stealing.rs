use crate::engine::task::job::TaskJob;
use crate::engine::task::worker::local_queue::LocalQueue;
use std::sync::{Arc, Mutex};

/// 轻量级 Work Stealing
pub struct WorkStealer {
    queues: Vec<Arc<Mutex<LocalQueue>>>,
}

impl WorkStealer {
    pub fn new(queues: Vec<Arc<Mutex<LocalQueue>>>) -> Self {
        Self { queues }
    }

    /// 从其他 Worker 偷一个任务（从尾部偷）
    pub fn steal(&self, my_index: usize) -> Option<TaskJob> {
        let len = self.queues.len();
        let start = (my_index + 1) % len;
        for i in 0..len {
            let idx = (start + i) % len;
            if idx == my_index {
                continue;
            }
            if let Some(job) = self.queues[idx].lock().unwrap().steal() {
                return Some(job);
            }
        }
        None
    }
}
