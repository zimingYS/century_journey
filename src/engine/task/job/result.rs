//! 定义通用异步任务的完成结果。

/// Task 执行结果
#[derive(Debug, Clone)]
pub enum TaskResult {
    /// 成功
    Success,
    /// 失败（含错误信息）
    Failed(String),
}
