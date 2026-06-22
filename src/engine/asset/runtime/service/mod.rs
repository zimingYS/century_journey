use crate::engine::asset::runtime::context::RuntimeContext;

/// Runtime Service trait
///
/// 所有 Runtime 服务（Streaming/Reload/Memory/Reference/Diagnostics）
/// 必须实现此 Trait。Runtime 通过 Scheduler 统一调度。
pub trait RuntimeService: Send + Sync + 'static {
    /// 服务名称
    fn name(&self) -> &str;

    /// 启动时初始化
    fn startup(&mut self, _ctx: &mut RuntimeContext) {}

    /// 每帧更新（由 RuntimeScheduler 调用）
    fn update(&mut self, ctx: &mut RuntimeContext);

    /// 关闭时清理
    fn shutdown(&mut self, _ctx: &mut RuntimeContext) {}
}
