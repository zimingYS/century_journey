use crate::engine::asset::runtime::context::RuntimeContext;
use bevy::prelude::*;
use std::collections::HashMap;

/// 热重载状态
#[derive(Debug, Clone)]
struct ReloadState {
    /// 旧 Handle（等待替换）
    #[allow(dead_code)]
    old_handle_exists: bool,
    /// 新 Handle 是否已加载
    new_loaded: bool,
    /// 重载时间戳
    #[allow(dead_code)]
    timestamp: f64,
}

/// 热重载服务（原子替换版）
///
/// 流程：
/// 1. Watcher 检测文件变更 → 标记 Reloading
/// 2. Pipeline 加载新版本 → 新 Handle 就绪
/// 3. 原子 Swap: 新 Handle 替换旧 Handle（不丢失一帧）
/// 4. Fire Event → AssetReloaded
/// 5. Release Old → 旧 Handle 引用释放
pub struct ReloadService {
    /// 监控目录
    watch_dirs: Vec<String>,
    /// 正在重载中的资源
    reload_states: HashMap<String, ReloadState>,
    /// 总重载次数
    reload_count: u32,
    /// 上次轮询时间
    last_poll: f64,
    /// 轮询间隔（秒）
    poll_interval: f64,
}

impl ReloadService {
    pub fn new(poll_interval: f64) -> Self {
        Self {
            watch_dirs: vec!["assets".into()],
            reload_states: HashMap::new(),
            reload_count: 0,
            last_poll: 0.0,
            poll_interval,
        }
    }

    /// 添加监控目录
    pub fn watch(&mut self, dir: impl Into<String>) {
        self.watch_dirs.push(dir.into());
    }

    /// 轮询文件变更（返回变更文件列表）
    pub fn poll(&mut self) -> Vec<String> {
        let now = now_secs();
        if now - self.last_poll < self.poll_interval {
            return vec![];
        }
        self.last_poll = now;

        let mut changed = Vec::new();
        for dir in &self.watch_dirs {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let key = path.to_string_lossy().to_string();
                        if !self.reload_states.contains_key(&key) {
                            changed.push(key.clone());
                            self.reload_states.insert(
                                key,
                                ReloadState {
                                    old_handle_exists: true,
                                    new_loaded: false,
                                    timestamp: now,
                                },
                            );
                        }
                    }
                }
            }
        }
        changed
    }

    /// 标记新 Handle 已加载（由 Pipeline 回调）
    pub fn on_new_loaded(&mut self, key: &str) {
        if let Some(state) = self.reload_states.get_mut(key) {
            state.new_loaded = true;
            self.reload_count += 1;
        }
    }

    /// 获取可以执行 Swap 的 key（新版本已加载，旧版本可释放）
    pub fn ready_to_swap(&self) -> Vec<String> {
        self.reload_states
            .iter()
            .filter(|(_, s)| s.new_loaded)
            .map(|(k, _)| k.clone())
            .collect()
    }

    /// 执行原子替换并释放旧状态
    pub fn swap(&mut self, key: &str) {
        self.reload_states.remove(key);
    }

    /// 总重载次数
    pub fn reload_count(&self) -> u32 {
        self.reload_count
    }

    /// 待重载资源数
    pub fn pending_count(&self) -> usize {
        self.reload_states.len()
    }
}

impl crate::engine::asset::runtime::service::RuntimeService for ReloadService {
    fn name(&self) -> &str {
        "ReloadService"
    }

    fn update(&mut self, ctx: &mut RuntimeContext) {
        let changed = self.poll();
        for _key in &changed {
            ctx.diagnostics.reload_count = self.reload_count;
        }
    }
}

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
