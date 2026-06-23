use crate::engine::task::cancellation::manager::CancellationManager;
use crate::engine::task::dependency::graph::DependencyGraph;
use crate::engine::task::job::TaskJob;
use crate::engine::task::scheduler::budget::FrameBudget;

/// Scheduler Dispatch Pipeline
/// 每帧对候选任务依次执行：依赖检查 → 取消检查 → 预算检查 → Worker 选择 → 派发。
pub struct DispatchPipeline;

impl DispatchPipeline {
    /// 判断一个任务是否可以派发
    pub fn evaluate(
        job: &TaskJob,
        budget: &FrameBudget,
        graph: &DependencyGraph,
        cancellation: &CancellationManager,
    ) -> DispatchDecision {
        // 依赖检查
        if job.dependencies.has_dependencies() && !job.dependencies.all_resolved() {
            return DispatchDecision::WaitingDependency;
        }
        if !graph.all_resolved(job.id.value()) {
            return DispatchDecision::WaitingDependency;
        }

        // 取消检查
        if cancellation.is_cancelled(job.id) {
            return DispatchDecision::Cancelled;
        }

        // 预算检查
        if !budget.can_dispatch(job.priority) {
            return DispatchDecision::BudgetExceeded;
        }

        // 通过
        DispatchDecision::Ready
    }
}

/// Dispatch 决策结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchDecision {
    /// 可以派发
    Ready,
    /// 等待依赖
    WaitingDependency,
    /// 已取消
    Cancelled,
    /// 超出预算
    BudgetExceeded,
}
