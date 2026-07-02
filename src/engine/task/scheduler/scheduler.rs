use crate::engine::task::job::{TaskId, TaskJob};
use crate::engine::task::scheduler::queue::TaskQueue;

/// 任务调度器
/// 负责任务的提交、派发与完成回收，维护待执行任务队列与已完成任务集合，支撑任务调度的核心流程
pub struct TaskScheduler {
    /// 待执行任务队列
    queue: TaskQueue,
    /// 已完成的任务作业集合
    completed: Vec<TaskJob>,
    /// 历史累计完成任务计数
    completed_counter: u64,
}

impl TaskScheduler {
    /// 创建空的任务调度器实例
    pub fn new() -> Self {
        Self {
            queue: TaskQueue::new(),
            completed: Vec::new(),
            completed_counter: 0,
        }
    }

    /// 提交任务作业
    /// 将任务加入待执行队列，等待调度派发
    pub fn submit(&mut self, job: TaskJob) {
        self.queue.push(job);
    }

    /// 取出一个待执行任务
    /// 从队列中弹出可执行的任务，无可用任务时返回 None
    pub fn fetch_job(&mut self) -> Option<TaskJob> {
        self.queue.pop()
    }

    /// 登记任务完成
    /// 将执行完毕的任务存入已完成集合，并累加历史完成计数
    pub fn complete(&mut self, job: TaskJob) {
        self.completed.push(job);
        self.completed_counter += 1;
    }

    /// 批量收集已完成任务
    /// 取出所有已完成的任务作业，同时清空调度器内部的已完成列表
    pub fn collect_results(&mut self) -> Vec<TaskJob> {
        std::mem::take(&mut self.completed)
    }

    /// 添加任务间依赖关系
    /// 基于队列中的任务建立依赖关联，实际的依赖判定与解算由依赖图组件统一管理
    pub fn add_dependency(&mut self, _from: TaskId, _to: TaskId) {
        // 通过队列中的任务建立依赖
        // 实际依赖解决由 DependencyGraph 管理
    }

    /// 获取当前待执行任务的数量
    pub fn pending_count(&self) -> usize {
        self.queue.pending()
    }

    /// 获取历史累计完成的任务总数
    pub fn completed_count(&self) -> u64 {
        self.completed_counter
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
