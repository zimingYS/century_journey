//! 主菜单世界目录的扫描、排序与安全标识生成。

use std::time::{SystemTime, UNIX_EPOCH};

use bevy::prelude::*;

use super::contracts::{WorldCatalog, WorldSummary};
use crate::game::save::world::metadata::io;

/// 在进入主菜单时刷新世界目录资源。
pub(super) fn refresh_world_catalog_system(mut catalog: ResMut<WorldCatalog>) {
    refresh_world_catalog(&mut catalog);
}

/// 从存档目录重建世界摘要，并尽量保留原有选择。
pub(super) fn refresh_world_catalog(catalog: &mut WorldCatalog) {
    let previous = catalog.selected.clone();
    catalog.worlds.clear();
    let root = std::path::Path::new("saves");
    let Ok(entries) = std::fs::read_dir(root) else {
        catalog.selected = None;
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(id) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let data = match io::load_level(&id) {
            Ok(data) => data,
            Err(_) => match io::load_level_backup(&id) {
                Ok(data) => data,
                Err(_) => continue,
            },
        };
        let modified_unix = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |duration| duration.as_secs());
        catalog.worlds.push(WorldSummary {
            id,
            seed: data.seed,
            modified_unix,
        });
    }
    catalog
        .worlds
        .sort_by_key(|world| std::cmp::Reverse(world.modified_unix));
    catalog.selected = previous
        .filter(|selected| catalog.worlds.iter().any(|world| &world.id == selected))
        .or_else(|| catalog.worlds.first().map(|world| world.id.clone()));
}

/// 将玩家输入转换为只含安全 ASCII 字符的世界标识。
pub(super) fn sanitize_world_name(name: &str) -> String {
    let mut result = String::new();
    for character in name.trim().chars() {
        if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
            result.push(character.to_ascii_lowercase());
        } else if character.is_whitespace() && !result.ends_with('_') {
            result.push('_');
        }
    }
    result = result.trim_matches('_').to_string();
    if result.is_empty() {
        format!(
            "world_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        )
    } else {
        result
    }
}

/// 在已有目录后追加最小可用数字后缀，避免覆盖世界。
pub(super) fn unique_world_id(base: &str, catalog: &WorldCatalog) -> String {
    if !catalog.worlds.iter().any(|world| world.id == base) {
        return base.to_string();
    }
    (2..)
        .map(|suffix| format!("{base}_{suffix}"))
        .find(|candidate| !catalog.worlds.iter().any(|world| &world.id == candidate))
        .expect("world suffix space is effectively unbounded")
}

/// 校验世界标识可安全用作当前存档目录名。
pub(super) fn valid_world_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

#[cfg(test)]
#[path = "../../../tests/unit/app/flow/catalog.rs"]
mod tests;
