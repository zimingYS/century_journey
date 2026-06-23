/// Task Runtime 运行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    /// 未启动
    Stopped,
    /// 运行中
    Running,
    /// 关闭中
    ShuttingDown,
}
