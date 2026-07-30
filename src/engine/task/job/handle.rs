//! 定义调用方用于观察任务状态的轻量句柄。

use crate::engine::task::job::id::TaskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// 调用方持有的异步任务标识句柄。
pub struct TaskHandle {
    id: TaskId,
}

impl TaskHandle {
    /// 使用给定参数创建新实例。
    pub(crate) fn new(id: TaskId) -> Self {
        Self { id }
    }

    /// 返回该句柄关联的任务标识。
    pub fn id(&self) -> TaskId {
        self.id
    }
}
