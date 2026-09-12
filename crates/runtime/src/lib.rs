//! cj-runtime —— sim ⇄ ECS 适配层
//!
//! 规则：唯一允许把 simulation 接到 ECS 的地方（§6.1）。

use bevy::prelude::*;

pub mod world_data;

/// 把 sim / world 接进 Bevy App
pub struct RuntimePlugin;

impl Plugin for RuntimePlugin {
    fn build(&self, _app: &mut App) {
        // TODO: 注册模拟调度（§6.7 频率分层）
    }
}
