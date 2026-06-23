use crate::engine::task::job::id::TaskId;
use std::collections::HashSet;

/// Task 依赖关系（附加到 TaskJob 上）
#[derive(Debug, Clone, Default)]
pub struct TaskDependency {
    /// 本任务依赖的其他任务 ID
    pub depends_on: HashSet<TaskId>,
    /// 依赖本任务的其他任务 ID
    pub depended_by: HashSet<TaskId>,
    /// 已完成的依赖数量
    pub resolved_count: u32,
    /// 总依赖数量
    pub total_count: u32,
}

impl TaskDependency {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加依赖
    pub fn add(&mut self, id: TaskId) {
        self.depends_on.insert(id);
        self.total_count = self.depends_on.len() as u32;
    }

    /// 标记一个依赖已解决
    pub fn resolve_one(&mut self) {
        self.resolved_count += 1;
    }

    /// 所有依赖是否已满足
    pub fn all_resolved(&self) -> bool {
        self.resolved_count >= self.total_count
    }

    /// 是否有依赖
    pub fn has_dependencies(&self) -> bool {
        self.total_count > 0
    }
}
