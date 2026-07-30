//! 内容文件路径、定义标识符和标签标识符的统一解析规则。
//!
//! 路径规则集中于此，避免不同领域对命名空间和 Windows 分隔符作出不同解释。

use crate::engine::asset::AssetResolver;
use crate::shared::identifier::Identifier;
use crate::shared::tag::identifier::TagId;

/// 按内容来源优先级判断相对路径是否由任一来源提供。
pub(super) fn content_file_exists(resolver: &AssetResolver, relative: &str) -> bool {
    resolver
        .content_roots()
        .iter()
        .rev()
        .any(|root| root.join(relative).is_file())
}

/// 从配方定义路径推导稳定配方标识符。
pub(super) fn recipe_id(path: &str) -> Option<Identifier> {
    definition_identifier(path, "definitions/recipes/", None)
}

/// 从方块掉落表路径推导目标方块标识符。
pub(super) fn block_loot_id(path: &str) -> Option<Identifier> {
    definition_identifier(path, "definitions/loot/blocks/", Some("century_journey"))
}

/// 按目录前缀和可选默认命名空间解析定义标识符。
fn definition_identifier(
    path: &str,
    prefix: &str,
    default_namespace: Option<&str>,
) -> Option<Identifier> {
    let relative = path.strip_prefix(prefix)?.replace('\\', "/");
    if let Some((namespace, name)) = relative.split_once('/') {
        (!namespace.is_empty() && !name.is_empty()).then(|| Identifier::new(namespace, name))
    } else {
        default_namespace.map(|namespace| Identifier::new(namespace, relative))
    }
}

/// 从带领域层级的标签路径提取运行时标签标识符。
pub(super) fn tag_runtime_id(path: &str) -> Option<TagId> {
    let relative = path.strip_prefix("definitions/tags/")?.replace('\\', "/");
    let mut parts = relative.split('/');
    let _kind = parts.next()?;
    let namespace = parts.next()?;
    let name = parts.collect::<Vec<_>>().join("/");
    (!namespace.is_empty() && !name.is_empty()).then(|| TagId::new(namespace, name))
}

/// 将方块或物品定义中的内联标签写法规范化为标签标识符。
pub(super) fn inline_tag_id(tag: &str) -> TagId {
    if let Some((namespace, path)) = tag.split_once('/') {
        TagId::new(namespace, path)
    } else {
        TagId::new("century_journey", tag)
    }
}

/// 保留标签领域并生成用于交叉引用校验的完整身份字符串。
pub(super) fn tag_identity(path: &str) -> Option<String> {
    let relative = path.strip_prefix("definitions/tags/")?;
    let mut parts = relative.split('/');
    let kind = parts.next()?;
    let namespace = parts.next()?;
    let name = parts.collect::<Vec<_>>().join("/");
    (!name.is_empty()).then(|| format!("{kind}:{namespace}:{name}"))
}

/// 将定义相对路径转换为命名空间与资源路径组合。
pub(super) fn definition_id(path: &str, prefix: &str) -> String {
    let relative = path.strip_prefix(prefix).unwrap_or(path);
    relative
        .split_once('/')
        .map(|(namespace, name)| format!("{namespace}:{name}"))
        .unwrap_or_else(|| format!("century_journey:{relative}"))
}
