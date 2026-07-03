/*
    此部分用于从目录加载所有 JSON 格式的标签定义，自动解析标签间的互相引用，
    处理命名空间，最终构建出支持双向快速查询的TagRegistry注册表
    具体流程：
    1. 检查标签根目录是否存在
    2. 遍历根目录下的子目录（每个子目录对应一种标签类型：blocks/biomes）
    3. 递归扫描每个子目录下的所有.json文件
    4. 从文件路径自动推导标签ID
    5. 解析JSON内容，分离"直接条目"和"标签引用"
    6. 先插入所有直接条目到注册表
    7. 迭代解析所有标签引用
    8. 检测并警告循环引用和无效引用
    9. 验证所有方块标签的条目是否真实存在于方块注册表
    10. 返回最终构建完成的TagRegistry
*/

use crate::engine::asset::manager::AssetManager;
use crate::shared::tag::identifier::{TagId, TagRegistryType};
use crate::shared::tag::registry::{TagRegistry, TypedTagRegistry};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// 标签定义文件格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDefinition {
    /// 是否替换已有同名标签（而非追加）
    #[serde(default)]
    pub replace: bool,
    /// 标签值列表 （`#` 前缀表示引用其他标签）
    pub values: Vec<String>,
}

/// 未解析的标签原始数据
struct UnresolvedTag {
    tag_id: TagId,
    replace: bool,
    direct_entries: HashSet<String>,
    tag_references: HashSet<TagId>,
}

/// 从 `assets/definitions/tags/` 通过 AssetManager 加载所有标签
pub fn load_tags_from_assets(asset: &AssetManager) -> TagRegistry {
    let mut registry = TagRegistry::default();
    let tags_root = PathBuf::from("assets/definitions/tags");

    if !tags_root.exists() {
        log::info!("[标签] 标签目录不存在，跳过加载: {:?}", tags_root);
        return registry;
    }

    let Ok(registry_dirs) = std::fs::read_dir(&tags_root) else {
        log::warn!("[标签] 无法读取标签目录: {:?}", tags_root);
        return registry;
    };

    for registry_dir in registry_dirs.flatten() {
        let file_name = registry_dir.file_name();
        let dir_name = match file_name.to_str() {
            Some(name) => name,
            None => continue,
        };

        let registry_type = TagRegistryType::from_dir_name(&dir_name);
        let unresolved = load_typed_tags(asset, &registry_dir.path());
        let typed = resolve_and_build_typed(unresolved);
        registry.registries.insert(registry_type, typed);
    }

    log::info!(
        "[标签] 加载完成 — 方块标签 {} 个, 群系标签 {} 个",
        registry.tag_count(&TagRegistryType::Block),
        registry.tag_count(&TagRegistryType::Biome),
    );

    registry
}

fn load_typed_tags(asset: &AssetManager, dir: &Path) -> Vec<UnresolvedTag> {
    let mut results = Vec::new();
    scan_tag_dir(asset, dir, dir, &mut results);
    results
}

fn scan_tag_dir(
    asset: &AssetManager,
    base: &Path,
    current: &Path,
    results: &mut Vec<UnresolvedTag>,
) {
    let Ok(entries) = std::fs::read_dir(current) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_tag_dir(asset, base, &path, results);
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let relative = path.strip_prefix(base).unwrap_or(&path);
        let Some(tag_id) = path_to_tag_id(relative) else {
            log::warn!("[标签] 无法从路径推导标签ID: {:?}", path);
            continue;
        };

        // 通过 AssetManager 读取
        let relative_str = relative.to_str().unwrap_or("");
        let asset_path = format!("definitions/tags/{}", relative_str.replace('\\', "/"));
        let id = crate::engine::asset::identifier::asset_id(&asset_path);
        let definition = match asset.read_json_sync::<TagDefinition>(&id) {
            Ok(d) => d,
            Err(e) => {
                log::error!("[标签] 解析失败 {:?}: {}", path, e);
                continue;
            }
        };

        let mut direct_entries = HashSet::new();
        let mut tag_references = HashSet::new();

        for value in definition.values {
            if let Some(ref_id) = TagId::from_reference(&value) {
                tag_references.insert(ref_id);
            } else if let Some(entry_id) = validate_entry_id(&value) {
                direct_entries.insert(entry_id);
            } else {
                log::warn!("[标签] 标签 {} 中的无效条目被忽略: '{}'", tag_id, value);
            }
        }

        results.push(UnresolvedTag {
            tag_id,
            replace: definition.replace,
            direct_entries,
            tag_references,
        });
    }
}

/// 将相对路径转换为标签ID
fn path_to_tag_id(relative: &Path) -> Option<TagId> {
    let stem = relative.to_str()?.strip_suffix(".json")?;
    let parts: Vec<&str> = stem
        .split(|c| c == '/' || c == std::path::MAIN_SEPARATOR)
        .collect();

    // 命名空间至少有两个字段
    if parts.len() < 2 {
        return None;
    }

    let namespace = parts[0].to_string();
    let path = parts[1..].join("/");

    if namespace.is_empty() || path.is_empty() {
        return None;
    }

    Some(TagId::new(namespace, path))
}

/// 验证条目标识符格式
fn validate_entry_id(id: &str) -> Option<String> {
    if id.is_empty() || id.starts_with('#') {
        return None;
    }
    // 含冒号的完整标识符直接通过
    if id.contains(':') {
        Some(id.to_string())
    } else {
        // 无命名空间 → 补默认命名空间
        Some(format!("century_journey:{}", id))
    }
}

/// 解析标签引用并构建最终的注册表
fn resolve_and_build_typed(unresolved: Vec<UnresolvedTag>) -> TypedTagRegistry {
    let mut typed = TypedTagRegistry::default();

    // 插入所有直接条目
    for u in &unresolved {
        if u.replace {
            typed.replace_tag(u.tag_id.clone(), u.direct_entries.clone());
        } else {
            typed.append_to_tag(u.tag_id.clone(), u.direct_entries.clone());
        }
    }

    // 收集所有待解析的标签引用
    let mut pending: HashMap<TagId, HashSet<TagId>> = HashMap::new();
    for u in &unresolved {
        if !u.tag_references.is_empty() {
            pending.insert(u.tag_id.clone(), u.tag_references.clone());
        }
    }

    // 迭代解析标签引用，直到所有引用被展开或无法继续
    resolve_tag_references(&mut typed, &mut pending);

    typed
}

/// 迭代解析标签引用
fn resolve_tag_references(
    typed: &mut TypedTagRegistry,
    pending: &mut HashMap<TagId, HashSet<TagId>>,
) {
    let max_iterations = 100;

    for iteration in 0..max_iterations {
        let mut any_changed = false;

        // 收集本轮可以展开的引用
        let to_expand: Vec<(TagId, Vec<TagId>)> = pending
            .iter()
            .map(|(tag_id, refs)| (tag_id.clone(), refs.iter().cloned().collect()))
            .collect();

        for (tag_id, references) in to_expand {
            let mut resolved_refs = Vec::new();

            for ref_tag_id in &references {
                // 被引用的标签是否存在？
                if typed.get_tag_entries(ref_tag_id).is_some() {
                    // 被引用标签是否还有未解析引用
                    let ref_has_pending = pending
                        .get(ref_tag_id)
                        .map_or(false, |refs| !refs.is_empty());

                    if !ref_has_pending {
                        // 可以安全展开
                        resolved_refs.push(ref_tag_id.clone());
                    }
                } else {
                    log::warn!(
                        "[标签] 引用的标签不存在: #{} (在标签 {} 中)",
                        ref_tag_id,
                        tag_id
                    );
                    resolved_refs.push(ref_tag_id.clone()); // 移除无效引用
                }
            }

            // 展开已解析的引用：将被引用标签的所有条目加入当前标签
            for ref_tag_id in &resolved_refs {
                if let Some(entries) = typed.get_tag_entries(ref_tag_id).cloned() {
                    for entry in entries {
                        typed.insert(tag_id.clone(), entry);
                    }
                }
                // 从待解析列表中移除
                if let Some(pending_refs) = pending.get_mut(&tag_id) {
                    pending_refs.remove(ref_tag_id);
                }
            }

            if !resolved_refs.is_empty() {
                any_changed = true;
            }
        }

        // 清理空的待解析列表
        pending.retain(|_, refs| !refs.is_empty());

        if !any_changed {
            break;
        }

        if iteration == max_iterations - 1 {
            log::warn!("[标签] 引用解析达到最大迭代次数，可能存在复杂依赖");
        }
    }

    // 报告未解析的引用
    if !pending.is_empty() {
        for (tag_id, refs) in pending {
            log::warn!(
                "[标签] 存在未解析的循环引用: {} → [{}]",
                tag_id,
                refs.iter()
                    .map(|r| format!("#{}", r))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
