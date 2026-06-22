use crate::engine::asset::runtime::context::{DiagnosticsSnapshot, RuntimeContext};

/// 诊断服务
pub struct DiagnosticsService {
    /// 当前快照
    pub snapshot: DiagnosticsSnapshot,
}

impl DiagnosticsService {
    pub fn new() -> Self {
        Self {
            snapshot: DiagnosticsSnapshot::default(),
        }
    }

    /// 记录缓存命中
    pub fn record_cache_hit(&mut self) {
        self.snapshot.cache_hits += 1;
    }

    /// 记录缓存未命中
    pub fn record_cache_miss(&mut self) {
        self.snapshot.cache_misses += 1;
    }

    /// 记录加载时间
    pub fn record_load_time(&mut self, ms: f64) {
        let old = self.snapshot.avg_load_time_ms * self.snapshot.loaded_count as f64;
        self.snapshot.loaded_count += 1;
        self.snapshot.total_load_time_ms += ms;
        self.snapshot.avg_load_time_ms = (old + ms) / self.snapshot.loaded_count as f64;
    }

    /// 缓存命中率
    pub fn cache_hit_rate(&self) -> f32 {
        let total = self.snapshot.cache_hits + self.snapshot.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.snapshot.cache_hits as f32 / total as f32
        }
    }
}

impl crate::engine::asset::runtime::service::RuntimeService for DiagnosticsService {
    fn name(&self) -> &str {
        "DiagnosticsService"
    }

    fn update(&mut self, ctx: &mut RuntimeContext) {
        // 同步到 Context
        ctx.diagnostics = self.snapshot.clone();
    }
}
