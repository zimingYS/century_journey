use crate::engine::task::manager::TaskManager;
use crate::engine::task::runtime::TaskRuntimePlugin;
use crate::engine::task::scheduler::TaskScheduler;
use crate::engine::task::worker::WorkerPool;
use bevy::prelude::*;
use std::sync::{Arc, Mutex};

/// Task Plugin
/// 初始化 TaskManager + TaskScheduler + WorkerPool + RuntimeContext。
pub struct TaskPlugin;

impl Plugin for TaskPlugin {
    fn build(&self, app: &mut App) {
        // 创建Scheduler（线程安全共享）
        let scheduler = Arc::new(Mutex::new(TaskScheduler::new()));

        // 创建Manager（Facade，存入 Bevy Resource）
        let manager = TaskManager::new(scheduler.clone());

        // 启 WorkerPool（在后台线程中）
        let pool = WorkerPool::with_thread_count(scheduler);

        app.insert_resource(manager)
            .insert_non_send(pool)
            .add_plugins(TaskRuntimePlugin);
    }
}
