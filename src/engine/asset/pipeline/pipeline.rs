use crate::engine::asset::pipeline::context::AssetPipelineContext;
use crate::engine::asset::pipeline::request::AssetRequest;
use crate::engine::asset::pipeline::response::{AssetResponse, AssetResponseMetadata};
use crate::engine::asset::pipeline::stage::AssetStage;
use crate::engine::asset::state::AssetState;
use bevy::prelude::*;
use std::time::Instant;

/// Asset Pipeline
///
/// 管理一组有序的 Stage，按顺序执行。
/// 每个 Request 都走完整的 Pipeline。
#[derive(Resource)]
pub struct AssetPipeline {
    stages: Vec<Box<dyn AssetStage>>,
}

impl AssetPipeline {
    /// 创建空 Pipeline
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// 添加一个 Stage
    pub fn add_stage(mut self, stage: impl AssetStage) -> Self {
        self.stages.push(Box::new(stage));
        self
    }

    /// 执行 Pipeline 处理一个请求
    pub fn process(&self, request: AssetRequest) -> (AssetResponse, AssetPipelineContext) {
        let start = Instant::now();
        let mut ctx = AssetPipelineContext::new(request);

        for stage in &self.stages {
            if ctx.is_failed() {
                break;
            }

            if let Err(e) = stage.process(&mut ctx) {
                ctx.fail(format!("[{}] {}", stage.name(), e));
            }
        }

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let response_meta = AssetResponseMetadata {
            id: ctx.metadata.id.to_string(),
            asset_type: ctx.metadata.asset_type.clone(),
            load_time_ms: elapsed,
        };

        let response = if ctx.is_failed() {
            let err = ctx.metadata.last_error.clone().unwrap_or_default();
            let mut resp = AssetResponse::failed(&err, response_meta);
            resp.diagnostics = ctx.diagnostics.clone();
            resp
        } else {
            let mut resp = AssetResponse::success_empty(response_meta);
            resp.diagnostics = ctx.diagnostics.clone();
            resp
        };

        ctx.metadata.state = if response.success {
            AssetState::Ready
        } else {
            AssetState::Failed
        };

        (response, ctx)
    }
}

impl Default for AssetPipeline {
    fn default() -> Self {
        Self::new()
    }
}
