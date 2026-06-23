use crate::engine::task::job::TaskJob;
use crate::engine::task::scheduler::priority::TaskPriority;
use std::collections::VecDeque;

/// 五级优先级任务队列
///
/// 不使用 BinaryHeap，改用五个 FIFO VecDeque，
/// 保证同优先级内严格先进先出。
pub struct TaskQueue {
    critical: VecDeque<TaskJob>,
    high: VecDeque<TaskJob>,
    normal: VecDeque<TaskJob>,
    low: VecDeque<TaskJob>,
    idle: VecDeque<TaskJob>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            critical: VecDeque::new(),
            high: VecDeque::new(),
            normal: VecDeque::new(),
            low: VecDeque::new(),
            idle: VecDeque::new(),
        }
    }

    /// 按优先级入队
    pub fn push(&mut self, job: TaskJob) {
        match job.priority {
            TaskPriority::Critical => self.critical.push_back(job),
            TaskPriority::High => self.high.push_back(job),
            TaskPriority::Normal => self.normal.push_back(job),
            TaskPriority::Low => self.low.push_back(job),
            TaskPriority::Idle => self.idle.push_back(job),
        }
    }

    /// 按优先级出队（先取高优先级）
    pub fn pop(&mut self) -> Option<TaskJob> {
        if let Some(j) = self.critical.pop_front() {
            return Some(j);
        }
        if let Some(j) = self.high.pop_front() {
            return Some(j);
        }
        if let Some(j) = self.normal.pop_front() {
            return Some(j);
        }
        if let Some(j) = self.low.pop_front() {
            return Some(j);
        }
        self.idle.pop_front()
    }

    /// 队列中待处理的任务总数
    pub fn pending(&self) -> usize {
        self.critical.len() + self.high.len() + self.normal.len() + self.low.len() + self.idle.len()
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}
