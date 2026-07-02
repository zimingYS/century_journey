use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::pipeline::context::AssetPipelineContext;
use crate::engine::asset::pipeline::stage::AssetStage;
use crate::engine::asset::state::AssetState;
use std::path::PathBuf;

/// Resolver Stage: AssetId → 文件路径
pub struct ResolverStage {
    base_dir: String,
}

impl ResolverStage {
    pub fn new(base_dir: impl Into<String>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn resolve(&self, id: &AssetId) -> String {
        let s = id.to_string();
        let cleaned = s.split(':').last().unwrap_or(&s);
        let path = PathBuf::from(&self.base_dir).join(cleaned);
        path.to_string_lossy().to_string()
    }
}

impl AssetStage for ResolverStage {
    fn name(&self) -> &str {
        "Resolver"
    }

    fn process(&self, ctx: &mut AssetPipelineContext) -> Result<(), String> {
        let path = self.resolve(&ctx.request.id);
        ctx.resolved_path = Some(path.clone());
        ctx.metadata.source = format!("filesystem:{}", path);
        ctx.metadata.state = AssetState::Resolving;
        Ok(())
    }
}

/// Source Stage: 文件路径 → 原始字节
pub struct SourceStage;

impl AssetStage for SourceStage {
    fn name(&self) -> &str {
        "Source"
    }

    fn process(&self, ctx: &mut AssetPipelineContext) -> Result<(), String> {
        let path = ctx.resolved_path.as_ref().ok_or("no resolved path")?;
        let bytes = std::fs::read(path).map_err(|e| format!("read {path}: {e}"))?;
        ctx.raw_bytes = Some(bytes);
        ctx.metadata.file_size = Some(ctx.raw_bytes.as_ref().map(|b| b.len() as u64).unwrap_or(0));
        ctx.metadata.state = AssetState::Loading;
        Ok(())
    }
}

/// Cache Stage: 更新 Metadata 状态
pub struct CacheStage;

impl AssetStage for CacheStage {
    fn name(&self) -> &str {
        "Cache"
    }

    fn process(&self, ctx: &mut AssetPipelineContext) -> Result<(), String> {
        ctx.metadata.state = AssetState::Ready;
        Ok(())
    }
}
