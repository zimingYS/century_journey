/// 运行时统计 V3
#[derive(Debug, Clone, Default)]
pub struct RuntimeStatistics {
    pub pending: usize,
    pub running: usize,
    pub completed: u64,
    pub cancelled: u64,
    pub failed: u64,
    pub waiting_dependency: usize,
    pub avg_wait_us: u64,
    pub avg_execute_us: u64,
    pub worker_utilization: f32,
    pub queue_length: usize,
    pub cpu_queue: usize,
    pub io_queue: usize,
    pub dispatched_this_frame: usize,
    pub batch_count: u64,
    pub partition_time_us: u64,
    pub join_time_us: u64,
    pub barrier_wait_us: u64,
    pub steal_count: u64,
    pub avg_batch_size: usize,
    pub parallel_speedup: f32,
}
