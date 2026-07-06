use serde::{Deserialize, Serialize};
use std::fmt;

/// 统一资源标识符，格式：`namespace:path`。
///
/// 这是路径解析的唯一入口——任何模块要读资源，先构造 `AssetId`，
/// 再交给 [`AssetResolver`](super::resolver::AssetResolver)，不允许自己拼字符串路径。
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetId(String, String);

impl AssetId {
    pub fn new(namespace: impl Into<String>, path: impl Into<String>) -> Self {
        Self(namespace.into(), path.into())
    }

    pub fn namespace(&self) -> &str {
        &self.0
    }

    pub fn path(&self) -> &str {
        &self.1
    }

    /// 从 "namespace:path" 解析
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (ns, path) = raw
            .split_once(':')
            .ok_or_else(|| format!("AssetId must be 'namespace:path' format, got: {raw}"))?;
        Ok(Self(ns.into(), path.into()))
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.0, self.1)
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self("century_journey".into(), "".into())
    }
}

/// 使用默认命名空间 `century_journey` 构造 `AssetId`
pub fn asset_id(path: &str) -> AssetId {
    AssetId::new("century_journey", path)
}
