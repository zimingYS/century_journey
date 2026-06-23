use crate::engine::task::dependency::resolver::DependencyResolver;
use crate::engine::task::manager::TaskManager;
use crate::engine::task::runtime::context::RuntimeContext;
use bevy::prelude::*;

/// 每帧Runtime更新
pub fn task_runtime_update_system(task: Res<TaskManager>, mut ctx: ResMut<RuntimeContext>) {
    ctx.tick();

    // 收集已完成任务并解析依赖
    let completed = task.collect_results();
    for job in &completed {
        let _ready = DependencyResolver::resolve(&mut ctx.dependency_graph, job.id.value());
    }

    // 更新统计
    ctx.statistics.completed = task.completed_count();
    ctx.statistics.pending = task.pending_count();
    ctx.statistics.queue_length = task.pending_count();
    ctx.statistics.dispatched_this_frame = ctx.budget.dispatched_this_frame();
}
