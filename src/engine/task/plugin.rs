use crate::engine::task::manager::TaskManager;
use crate::engine::task::runtime::TaskRuntimePlugin;
use bevy::prelude::*;

/// Initializes the Bevy task-pool facade and runtime diagnostics.
pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TaskManager::new())
            .add_plugins(TaskRuntimePlugin);
    }
}
