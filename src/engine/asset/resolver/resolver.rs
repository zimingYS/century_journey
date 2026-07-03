use crate::engine::asset::identifier::AssetId;
use bevy::prelude::*;

/// 资源解析器 trait
///
/// 负责将 AssetId 解析为真实的资源路径。
/// 支持链式解析：Default → ResourcePack → Mod 覆盖。
pub trait AssetResolver: Send + Sync + 'static {
    /// 解析 AssetId → 路径，失败返回 None
    fn resolve(&self, id: &AssetId) -> Option<String>;

    /// 解析器名称
    fn name(&self) -> &str;
}

/// 默认解析器 — 使用标准 assets/ 目录布局
pub struct DefaultResolver {
    root: String,
}

impl DefaultResolver {
    pub fn new(root: impl Into<String>) -> Self {
        Self { root: root.into() }
    }
}

impl AssetResolver for DefaultResolver {
    fn resolve(&self, id: &AssetId) -> Option<String> {
        Some(format!("{}/{}.png", self.root, id.path()))
    }

    fn name(&self) -> &str {
        "DefaultResolver"
    }
}

/// 解析器链 — 按优先级依次尝试
#[derive(Resource)]
pub struct ResolverChain {
    resolvers: Vec<Box<dyn AssetResolver>>,
}

impl ResolverChain {
    pub fn new(resolvers: Vec<Box<dyn AssetResolver>>) -> Self {
        Self { resolvers }
    }

    /// 按优先级解析，返回第一个成功的结果
    pub fn resolve(&self, id: &AssetId) -> Option<String> {
        for r in &self.resolvers {
            if let Some(path) = r.resolve(id) {
                return Some(path);
            }
        }
        None
    }
}
