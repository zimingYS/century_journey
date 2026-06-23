use crate::engine::task::executor::barrier::TaskBarrier;
use crate::engine::task::job::TaskHandle;

/// 任务组
pub struct TaskGroup {
    handles: Vec<TaskHandle>,
    barrier: Option<TaskBarrier>,
}

impl TaskGroup {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
            barrier: None,
        }
    }

    /// 添加任务
    pub fn add(&mut self, handle: TaskHandle) {
        self.handles.push(handle);
    }

    /// 设置 Barrier（所有任务完成后才继续）
    pub fn with_barrier(&mut self) {
        self.barrier = Some(TaskBarrier::new(self.handles.len()));
    }

    /// 等待所有任务完成
    pub fn wait(&self) {
        if let Some(ref barrier) = self.barrier {
            barrier.wait();
        }
    }

    /// 任务数量
    pub fn len(&self) -> usize {
        self.handles.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.handles.is_empty()
    }
}

impl Default for TaskGroup {
    fn default() -> Self {
        Self::new()
    }
}
