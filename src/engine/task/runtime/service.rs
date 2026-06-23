/// RuntimeService Trait
///
/// 所有 Runtime 模块（Scheduler/Dependency/Cancellation/Executor/Diagnostics）
/// 统一实现此 Trait。由 RuntimeServices 统一管理生命周期。
pub trait RuntimeService: Send + Sync + 'static {
    fn name(&self) -> &str;
    fn startup(&mut self) {}
    fn update(&mut self) {}
    fn shutdown(&mut self) {}
}
