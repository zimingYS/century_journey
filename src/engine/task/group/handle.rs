use crate::engine::task::job::TaskHandle;

/// 任务组句柄
/// 用于批量管理多个任务句柄，统一持有一组任务的执行句柄集合，便于对批量任务进行统一管控
pub struct TaskGroupHandle {
    /// 内部维护的任务句柄列表
    handles: Vec<TaskHandle>,
}

impl TaskGroupHandle {
    /// 创建任务组句柄
    pub fn new(handles: Vec<TaskHandle>) -> Self {
        Self { handles }
    }

    /// 获取当前任务组包含的任务句柄总数
    pub fn len(&self) -> usize {
        self.handles.len()
    }
}