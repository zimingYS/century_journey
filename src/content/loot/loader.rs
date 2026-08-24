//! 从内容资产加载并校验掉落表定义。

use crate::content::format::load_versioned_json_dir;
use crate::content::loot::table::LootTable;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use std::collections::HashMap;

/// 从 `assets/definitions/loot/blocks/` 加载所有方块掉落表
///
/// 目录结构:
///   loot/blocks/namespace/block_name.json → Identifier(namespace, "block_name")
///
/// 掉落表路径必须带命名空间目录，与配方等定义保持一致的层级约定。
///
/// 格式:
///   ```json
///   { "entries": [{"item": "century_journey:dirt", "min_count": 1, "max_count": 1, "chance": 1.0}] }
///   ```
///
/// 返回: HashMap<Identifier, LootTable> — 标识符到掉落表的映射
pub fn load_loot_tables(asset: &AssetManager) -> HashMap<Identifier, LootTable> {
    let files = AssetFiles::new(asset.resolver());
    let pairs = load_versioned_json_dir::<LootTable>(&files, "definitions/loot/blocks");
    let mut tables = HashMap::with_capacity(pairs.len());

    for (asset_path, table) in pairs {
        let Some(relative) = asset_path.strip_prefix("definitions/loot/blocks/") else {
            log::warn!("[掉落表] 跳过无效路径的掉落表: {}", asset_path);
            continue;
        };

        let relative = relative.replace('\\', "/");
        let Some((namespace, path)) = relative.split_once('/') else {
            log::warn!("[掉落表] 掉落表路径缺少命名空间: {}", asset_path);
            continue;
        };

        tables.insert(Identifier::new(namespace, path.to_string()), table);
    }

    if tables.is_empty() {
        log::info!("[掉落表] 未加载到任何掉落表定义");
    } else {
        log::info!("[掉落表] 已加载 {} 个掉落表", tables.len());
    }

    tables
}
