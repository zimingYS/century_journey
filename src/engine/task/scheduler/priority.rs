/// Task 调度优先级
///
/// 数值越小优先级越高。Scheduler 永远先调度高优先级任务。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    /// 最高优先级（必须本帧完成）
    Critical = 0,
    /// 高优先级（重要但不阻塞）
    High = 1,
    /// 普通优先级（默认）
    Normal = 2,
    /// 低优先级（后台处理）
    Low = 3,
    /// 最低优先级（空闲时处理）
    Idle = 4,
}
