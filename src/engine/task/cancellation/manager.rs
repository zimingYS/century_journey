use crate::engine::task::cancellation::token::CancellationToken;
use crate::engine::task::job::id::TaskId;
use std::collections::HashMap;

/// 取消管理器
/// 维护所有活动任务的取消令牌映射。
pub struct CancellationManager {
    tokens: HashMap<u64, CancellationToken>,
}

impl CancellationManager {
    pub fn new() -> Self {
        Self {
            tokens: HashMap::new(),
        }
    }

    /// 注册任务的取消令牌
    pub fn register(&mut self, id: TaskId, token: CancellationToken) {
        self.tokens.insert(id.value(), token);
    }

    /// 取消指定任务
    pub fn cancel(&self, id: TaskId) {
        if let Some(token) = self.tokens.get(&id.value()) {
            token.cancel();
        }
    }

    /// 检查任务是否已取消
    pub fn is_cancelled(&self, id: TaskId) -> bool {
        self.tokens
            .get(&id.value())
            .map(|t| t.is_cancelled())
            .unwrap_or(false)
    }

    /// 移除任务
    pub fn remove(&mut self, id: TaskId) {
        self.tokens.remove(&id.value());
    }
}

impl Default for CancellationManager {
    fn default() -> Self {
        Self::new()
    }
}
