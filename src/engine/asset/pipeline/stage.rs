use crate::engine::asset::pipeline::context::AssetPipelineContext;

/// Pipeline Stage trait
///
/// 每个 Stage 只负责一种工作。
/// Stage 之间禁止直接通信，只能通过 `AssetPipelineContext` 传递数据。
///
/// 如果 Stage 返回 `Err`，Pipeline 会终止后续 Stage 的执行。
pub trait AssetStage: Send + Sync + 'static {
    /// Stage 名称（用于日志和诊断）
    fn name(&self) -> &str;

    /// 执行 Stage 逻辑
    fn process(&self, ctx: &mut AssetPipelineContext) -> Result<(), String>;
}
