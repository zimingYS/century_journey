use crate::engine::task::diagnostics::statistics::RuntimeStatistics;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct RuntimeContext {
    pub frame_tick: u64,
    pub statistics: RuntimeStatistics,
}

impl RuntimeContext {
    pub fn tick(&mut self) {
        self.frame_tick += 1;
    }
}
