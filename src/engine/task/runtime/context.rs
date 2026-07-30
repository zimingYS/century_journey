//! 保存任务运行时的帧级上下文与完成结果。

use crate::engine::task::diagnostics::statistics::RuntimeStatistics;
use bevy::prelude::*;

#[derive(Resource, Default)]
/// 保存任务运行时在当前帧观察到的统计状态。
pub struct RuntimeContext {
    pub frame_tick: u64,
    pub statistics: RuntimeStatistics,
}

impl RuntimeContext {
    /// 推进一次任务运行时统计刷新。
    pub fn tick(&mut self) {
        self.frame_tick += 1;
    }
}
