#[derive(Debug, Clone, Default)]
pub struct RuntimeStatistics {
    pub pending: usize,
    pub running: usize,
    pub completed: u64,
    pub failed: u64,
    pub worker_utilization: f32,
    pub queue_length: usize,
    pub dispatched_this_frame: usize,
}
