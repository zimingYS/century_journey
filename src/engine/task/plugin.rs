use crate::engine::task::manager::TaskManager;
use crate::engine::task::runtime::TaskRuntimePlugin;
use bevy::prelude::*;

/// 注册 Bevy 任务池门面和运行时统计资源。
pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TaskManager::new())
            .add_plugins(TaskRuntimePlugin);
    }
}
