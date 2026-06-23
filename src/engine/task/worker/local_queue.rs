use crate::engine::task::job::TaskJob;
use std::collections::VecDeque;

/// Worker 本地队列
/// 每个 Worker 优先从自己的 LocalQueue 取任务。
pub struct LocalQueue {
    queue: VecDeque<TaskJob>,
}

impl LocalQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    /// 推入本地队列
    pub fn push(&mut self, job: TaskJob) {
        self.queue.push_back(job);
    }

    /// 从本地队列取任务
    pub fn pop(&mut self) -> Option<TaskJob> {
        self.queue.pop_front()
    }

    /// 供 Work Stealing：从尾部偷一个
    pub fn steal(&mut self) -> Option<TaskJob> {
        self.queue.pop_back()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }
}

impl Default for LocalQueue {
    fn default() -> Self {
        Self::new()
    }
}
