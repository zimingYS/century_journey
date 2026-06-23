/// Worker 统计信息
#[derive(Debug, Clone, Default)]
pub struct WorkerStatistics {
    /// 已执行任务总数
    pub completed_count: u64,
    /// 总执行时间（微秒）
    pub total_execution_us: u64,
    /// 平均执行时间（微秒）
    pub avg_execution_us: u64,
    /// Worker 利用率（0.0 ~ 1.0）
    pub utilization: f32,
    /// CPU Worker 数量
    pub cpu_workers: usize,
    /// IO Worker 数量
    pub io_workers: usize,
}
