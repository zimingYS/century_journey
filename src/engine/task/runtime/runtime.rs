use crate::engine::task::runtime::context::RuntimeContext;
use crate::engine::task::runtime::update::task_runtime_update_system;
use bevy::prelude::*;

/// 任务运行时插件。
pub struct TaskRuntimePlugin;

impl Plugin for TaskRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RuntimeContext>()
            .add_systems(PostUpdate, task_runtime_update_system);
    }
}
