use crate::content::format::load_versioned_json_dir;
use crate::content::tag::definition::TagAction;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::tag::identifier::TagId;

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
    let files = AssetFiles::new(asset.resolver());
    let pairs = load_versioned_json_dir::<TagAction>(&files, "definitions/tags");

    let mut actions = Vec::with_capacity(pairs.len());

    for (asset_path, action) in pairs {
        // 剥离统一前缀，提取标签类型、命名空间与相对路径
        let Some(relative) = asset_path.strip_prefix("definitions/tags/") else {
            log::warn!("[标签] 跳过无效路径的标签定义: {}", asset_path);
            continue;
        };

        // 统一路径分隔符，兼容 Windows 反斜杠
        let relative = relative.replace('\\', "/");
        let parts: Vec<&str> = relative.split('/').collect();

        // 路径至少需要 类型/命名空间/文件名 三级，与原校验逻辑完全对齐
        if parts.len() < 3 {
            log::warn!("[标签] 标签路径层级不足，跳过: {}", asset_path);
            continue;
        }

        // 解析命名空间，对应原 path_to_tag_id 中 components[1] 的取值
        let namespace = parts[1].to_string();
        // 提取文件名主体（去除 .json 后缀）
        let filename = parts.last().unwrap();
        let stem = filename.strip_suffix(".json").unwrap_or(filename);

        // 拼接嵌套路径 + 文件名主体，生成 TagId 的路径段
        // 等价于原逻辑中 components[2..len-1] + stem 的拼接规则
        let mut path_parts: Vec<&str> = parts[2..parts.len() - 1].to_vec();
        path_parts.push(stem);
        let path_str = path_parts.join("/");

        let tag_id = TagId::new(namespace, path_str);
        log::info!("[标签] 加载: {} ({:?})", tag_id.to_full(), &action);
        actions.push((tag_id, action));
    }

    if actions.is_empty() {
        log::info!("[标签] 未加载到任何标签定义");
    }

    actions
}
