use crate::engine::asset::dependency::graph::DependencyGraph;
use bevy::prelude::*;

/// 依赖追踪器 — Bevy Resource 包装
#[derive(Resource, Default)]
pub struct DependencyTracker {
    pub graph: DependencyGraph,
}

impl DependencyTracker {
    pub fn add(
        &mut self,
        from: &crate::engine::asset::identifier::AssetId,
        to: &crate::engine::asset::identifier::AssetId,
    ) {
        self.graph.add_dependency(from, to);
    }
}
