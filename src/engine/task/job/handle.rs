use crate::engine::task::job::id::TaskId;
use crate::engine::task::scheduler::TaskScheduler;
use std::sync::{Arc, Mutex};

/// 业务层任务句柄
/// 用于在业务侧标识单个任务，可携带调度器引用以支持任务间的依赖关系声明
pub struct TaskHandle {
    /// 任务唯一标识 ID
    id: TaskId,
    /// 可选的调度器引用
    scheduler: Option<Arc<Mutex<TaskScheduler>>>,
}

impl TaskHandle {
    /// 创建仅包含任务ID的句柄
    /// 该实例不关联调度器，无法声明任务依赖
    pub(crate) fn new(id: TaskId) -> Self {
        Self {
            id,
            scheduler: None,
        }
    }

    /// 获取当前任务的唯一标识 ID
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// 声明当前任务依赖于目标任务
    /// 若当前句柄关联了调度器，则会将依赖关系注册到调度器中；未关联调度器时该方法无效果
    pub fn depends_on(&self, target: &TaskHandle) {
        if let Some(ref sched) = self.scheduler {
            let mut s = sched.lock().unwrap();
            s.add_dependency(self.id, target.id);
        }
    }
}
