use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// 取消令牌
/// Worker 在执行过程中通过 `token.is_cancelled()` 检测是否被取消。
/// 若取消，Worker 应立即退出。
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 检查是否已取消
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    /// 触发取消
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    /// 获取内部 Arc（供 clone 用）
    pub(crate) fn inner(&self) -> Arc<AtomicBool> {
        self.cancelled.clone()
    }

    /// 从内部 Arc 构造（供 CancellationSource 用）
    pub(crate) fn from_inner(inner: Arc<AtomicBool>) -> Self {
        Self { cancelled: inner }
    }
}
