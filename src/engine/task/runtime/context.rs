use crate::engine::task::cancellation::manager::CancellationManager;
use crate::engine::task::dependency::graph::DependencyGraph;
use crate::engine::task::diagnostics::statistics::RuntimeStatistics;
use crate::engine::task::scheduler::budget::FrameBudget;
use bevy::prelude::*;

/// 共享上下文
#[derive(Resource, Default)]
pub struct RuntimeContext {
    pub frame_tick: u64,
    /// 依赖图
    pub dependency_graph: DependencyGraph,
    /// 取消管理器
    pub cancellation: CancellationManager,
    /// 帧预算
    pub budget: FrameBudget,
    /// 运行时统计
    pub statistics: RuntimeStatistics,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self {
            frame_tick: 0,
            dependency_graph: DependencyGraph::new(),
            cancellation: CancellationManager::new(),
            budget: FrameBudget::default(),
            statistics: RuntimeStatistics::default(),
        }
    }

    pub fn tick(&mut self) {
        self.frame_tick += 1;
        self.budget.reset_frame();
    }
}
