//! 执行任务运行时每帧状态回收与统计更新。

use crate::engine::task::manager::TaskManager;
use crate::engine::task::runtime::context::RuntimeContext;
use bevy::prelude::*;

/// 刷新任务运行时统计并回收已完成状态。
pub fn task_runtime_update_system(task: Res<TaskManager>, mut ctx: ResMut<RuntimeContext>) {
    ctx.tick();

    ctx.statistics.completed = task.completed_count();
    ctx.statistics.failed = task.failed_count();
    ctx.statistics.pending = task.pending_count();
    ctx.statistics.running = task.running_count();
    ctx.statistics.queue_length = task.pending_count();
    ctx.statistics.dispatched_this_frame = 0;
    ctx.statistics.worker_utilization = if task.worker_count() == 0 {
        0.0
    } else {
        task.running_count() as f32 / task.worker_count() as f32
    };
}
