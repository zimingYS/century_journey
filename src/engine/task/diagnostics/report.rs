use crate::engine::task::diagnostics::statistics::RuntimeStatistics;

/// 运行时报告
pub struct TaskReport;

impl TaskReport {
    /// 生成报告
    pub fn generate(stats: &RuntimeStatistics) -> String {
        format!(
            "Task Report:\n\
             pending={} running={} completed={} cancelled={} failed={} waiting_dep={}\n\
             avg_wait={}us avg_exec={}us worker_util={:.2} queue={} dispatched={}\n\
             cpu_queue={} io_queue={}",
            stats.pending,
            stats.running,
            stats.completed,
            stats.cancelled,
            stats.failed,
            stats.waiting_dependency,
            stats.avg_wait_us,
            stats.avg_execute_us,
            stats.worker_utilization,
            stats.queue_length,
            stats.dispatched_this_frame,
            stats.cpu_queue,
            stats.io_queue,
        )
    }
}
