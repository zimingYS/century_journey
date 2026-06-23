use crate::engine::task::runtime::service::RuntimeService;

/// 统一 Runtime Service 管理器
///
/// 管理所有 RuntimeService 的生命周期。
pub struct RuntimeServices {
    services: Vec<Box<dyn RuntimeService>>,
}

impl RuntimeServices {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// 注册一个 Service
    pub fn register(&mut self, service: impl RuntimeService) {
        self.services.push(Box::new(service));
    }

    /// 启动所有 Service
    pub fn startup_all(&mut self) {
        for s in &mut self.services {
            s.startup();
        }
    }

    /// 更新所有 Service
    pub fn update_all(&mut self) {
        for s in &mut self.services {
            s.update();
        }
    }

    /// 关闭所有 Service
    pub fn shutdown_all(&mut self) {
        for s in &mut self.services {
            s.shutdown();
        }
    }
}

impl Default for RuntimeServices {
    fn default() -> Self {
        Self::new()
    }
}
