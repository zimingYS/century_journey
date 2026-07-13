/// 当前任务运行时的轻量统计快照。
#[derive(Debug, Clone, Default)]
pub struct RuntimeStatistics {
    /// 等待执行的任务数量。
    pub pending: usize,
    /// 正在执行的任务数量。
    pub running: usize,
    /// 已完成任务总数。
    pub completed: u64,
    /// 执行失败任务总数。
    pub failed: u64,
    /// 工作线程利用率，取值范围为 0.0 到 1.0。
    pub worker_utilization: f32,
    /// 当前全局队列长度。
    pub queue_length: usize,
    /// 当前帧已派发的任务数量。
    pub dispatched_this_frame: usize,
}
