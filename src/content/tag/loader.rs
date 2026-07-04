use crate::content::tag::definition::TagAction;
use crate::engine::asset::manager::AssetManager;
use crate::shared::tag::identifier::TagId;
use std::path::PathBuf;

/// 从 `assets/definitions/tags/` 加载所有 TagAction
///
/// 目录结构:
///   tags/{type}/namespace/path.json → TagId(namespace, "path")
///   tags/{type}/namespace/nested/path.json → TagId(namespace, "nested/path")
///
/// 文件内容: TagAction (append / remove / replace)
///
/// 返回: Vec<(TagId, TagAction)> — tag_id 和目标操作
pub fn load_tag_actions(asset: &AssetManager) -> Vec<(TagId, TagAction)> {
    let tags_root = PathBuf::from("assets/definitions/tags");

    if !tags_root.exists() {
        log::info!("[标签] 标签目录不存在，跳过加载: {:?}", tags_root);
        return Vec::new();
    }

    let Ok(registry_dirs) = std::fs::read_dir(&tags_root) else {
        log::warn!("[标签] 无法读取标签目录: {:?}", tags_root);
        return Vec::new();
    };

    let mut actions = Vec::new();

    for registry_dir in registry_dirs.flatten() {
        if !registry_dir.path().is_dir() {
            continue;
        }
        // 收集该类型目录下的所有标签定义
        collect_tag_files(asset, &registry_dir.path(), &mut actions);
    }

    actions
}

/// 递归扫描目录，收集所有 .json 标签文件
fn collect_tag_files(
    asset: &AssetManager,
    dir: &std::path::Path,
    actions: &mut Vec<(TagId, TagAction)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            // 递归扫描子目录（深度路径 → TagId path 部分用 / 连接）
            collect_tag_files(asset, &path, actions);
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        // 从文件路径推导 TagId
        let Some(tag_id) = path_to_tag_id(&path) else {
            continue;
        };

        // 通过 AssetManager 读取 JSON
        let asset_path = format!(
            "definitions/tags/{}",
            path.strip_prefix("assets/definitions/tags/")
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/")
        );

        let asset_id = crate::engine::asset::identifier::asset_id(&asset_path);

        match asset.read_json_sync::<TagAction>(&asset_id) {
            Ok(action) => {
                log::info!("[标签] 加载: {} ({:?})", tag_id.to_full(), &action);
                actions.push((tag_id, action));
            }
            Err(e) => {
                log::error!("[标签] JSON 加载失败 {:?}: {}", path, e);
            }
        }
    }
}

/// 从文件路径推导 TagId
///
/// 路径格式: tags/{type}/namespace/path.json (或嵌套子目录)
/// → TagId(namespace, "path") 或 TagId(namespace, "nested/path")
fn path_to_tag_id(path: &std::path::Path) -> Option<TagId> {
    // 跳过 tags root + type_dir 两级
    let components: Vec<&str> = path
        .iter()
        .rev()
        .take_while(|c| *c != "tags")
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .filter_map(|c| c.to_str())
        .collect();

    if components.len() < 3 {
        // 至少需要 type_dir/namespace/path.json
        return None;
    }

    // components = [type_dir, namespace, ...paths..., filename]
    // e.g. ["block", "century_journey", "solid.json"]
    let namespace = components[1].to_string();
    let stem = path.file_stem()?.to_str()?;

    // 路径部分：components[2..-1] + stem
    let mut path_parts: Vec<&str> = components[2..components.len() - 1].to_vec();
    path_parts.push(stem);

    let path_str = path_parts.join("/");
    Some(TagId::new(namespace, path_str))
}
