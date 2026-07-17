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
///   loot/blocks/block_name.json → Identifier("century_journey", "block_name")
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
            log::warn!("[Loot] 跳过无效路径的掉落表: {}", asset_path);
            continue;
        };

        let relative = relative.replace('\\', "/");
        let id = if let Some((namespace, path)) = relative.split_once('/') {
            Identifier::new(namespace, path.to_string())
        } else {
            Identifier::new("century_journey", relative)
        };
        log::info!("[Loot] 加载 {} ({} entries)", id, table.entries.len());
        tables.insert(id, table);
    }

    tables
}
