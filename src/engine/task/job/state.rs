/// Task 生命周期状态
///
/// Created → Queued → WaitingDependency → Ready → Running
///                                                   ├── Completed → Collected
///                                                   ├── Failed
///                                                   └── Cancelled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    /// 已创建
    Created,
    /// 已入队列
    Queued,
    /// 等待依赖完成
    WaitingDependency,
    /// 依赖满足，可执行
    Ready,
    /// 正在 Worker 线程中执行
    Running,
    /// 执行成功完成
    Completed,
    /// 执行失败
    Failed,
    /// 已被取消
    Cancelled,
    /// 结果已被取走
    Collected,
}
