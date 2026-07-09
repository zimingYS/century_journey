use bevy::prelude::*;

use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use crate::shared::identifier::identifier::DEFAULT_NAMESPACE;

use super::definition::ItemModelDefinition;
use super::registry::ItemModelRegistry;

/// 从 assets/models/items 加载物品模型 JSON。
pub fn load_item_models_system(mut registry: ResMut<ItemModelRegistry>, asset: Res<AssetManager>) {
    let files = AssetFiles::new(asset.resolver());
    let mut count = 0usize;

    for (path, mut model) in files.read_json_dir::<ItemModelDefinition>("models/items") {
        let identifier = model
            .identifier
            .clone()
            .unwrap_or_else(|| identifier_from_model_path(&path));
        model.identifier = Some(identifier.clone());
        registry.register(identifier, model);
        count += 1;
    }

    info!("[item model registry] loaded {count} item model definitions");
}

/// 根据模型文件路径推导模型 ID。
fn identifier_from_model_path(asset_path: &str) -> Identifier {
    let local = asset_path
        .strip_prefix("models/items/")
        .unwrap_or(asset_path)
        .replace('\\', "/");

    if let Ok(identifier) = Identifier::parse(&local) {
        return identifier;
    }

    if let Some((namespace, path)) = local.split_once('/')
        && !namespace.is_empty()
        && !path.is_empty()
    {
        return Identifier::new(namespace, path);
    }

    Identifier::new(DEFAULT_NAMESPACE, local)
}
