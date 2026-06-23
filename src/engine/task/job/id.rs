use std::sync::atomic::{AtomicU64, Ordering};

/// 全局唯一的 Task ID
///
/// 使用原子计数器生成单调递增的 ID。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    /// 生成新的唯一 TaskId
    pub fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// 获取内部数值
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}
