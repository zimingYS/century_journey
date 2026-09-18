//! cj-runtime —— sim ⇄ ECS 适配层
//!
//! 规则：唯一允许把 simulation 接到 ECS 的地方（§6.1）。

use crate::world_data::{WorldData, generate_initial_world};
use bevy::prelude::*;

pub mod world_data;

/// 把 sim / world 接进 Bevy App
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldData>()
            .add_systems(Startup, generate_initial_world);
    }
}
