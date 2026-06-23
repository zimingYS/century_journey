use crate::engine::task::dependency::graph::DependencyGraph;

/// 依赖解析器
///
/// 每次任务完成时调用，解锁依赖该任务的其他任务。
pub struct DependencyResolver;

impl DependencyResolver {
    /// 当一个任务完成时，解析依赖该任务的任务
    pub fn resolve(graph: &mut DependencyGraph, completed_id: u64) -> Vec<u64> {
        graph.mark_resolved(completed_id);
        let dependents = graph.dependents_of(completed_id);
        let mut ready = Vec::new();
        for dep_id in dependents {
            if graph.all_resolved(dep_id) {
                ready.push(dep_id);
            }
        }
        ready
    }
}
