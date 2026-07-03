use crate::shared::identifier::Identifier;
use crate::shared::identifier::identifier::DEFAULT_NAMESPACE;

/// 统一资源标识符 — 基于 `Identifier` 的类型别名。
///
/// 格式：`namespace:path`
pub type AssetId = Identifier;

/// 使用默认命名空间 `century_journey` 构造 `AssetId`。
pub fn asset_id(path: &str) -> AssetId {
    Identifier::new(DEFAULT_NAMESPACE, path)
}

/// 从 `namespace:path` 字符串解析 `AssetId`（失败时 panic）。
pub fn asset_id_parse(raw: &str) -> AssetId {
    Identifier::parse(raw)
        .unwrap_or_else(|_| panic!("AssetId must be 'namespace:path' format, got: {raw}"))
}

/// 安全解析 `AssetId`，失败返回 `None`。
pub fn asset_id_try_parse(raw: &str) -> Option<AssetId> {
    Identifier::parse(raw).ok()
}
