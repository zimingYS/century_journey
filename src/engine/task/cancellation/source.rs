use crate::engine::task::cancellation::token::CancellationToken;

/// 取消来源
/// 创建 `CancellationToken` 并管理其生命周期。
pub struct CancellationSource {
    token: CancellationToken,
}

impl Default for CancellationSource {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationSource {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
        }
    }

    /// 获取Token（供 Worker 使用）
    pub fn token(&self) -> CancellationToken {
        CancellationToken::from_inner(self.token.inner())
    }

    /// 触发取消
    pub fn cancel(&mut self) {
        self.token.cancel();
    }
}
