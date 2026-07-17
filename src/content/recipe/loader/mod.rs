use crate::content::format::load_versioned_json_dir;
use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::engine::asset::AssetFiles;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;

/// 从 assets/definitions/recipes 加载所有 RecipeDefinition
pub fn load_recipe_definitions(asset: &AssetManager) -> Vec<(Identifier, RecipeDefinition)> {
    let files = AssetFiles::new(asset.resolver());
    let pairs = load_versioned_json_dir::<RecipeDefinition>(&files, "definitions/recipes");
    let mut recipes = Vec::with_capacity(pairs.len());

    for (asset_path, recipe) in pairs {
        // 去掉统一前缀，提取命名空间与相对路径，与原逻辑等价
        let Some(relative) = asset_path.strip_prefix("definitions/recipes/") else {
            log::warn!("[Recipe] 跳过无效路径的配方: {}", asset_path);
            continue;
        };

        // 统一路径分隔符，兼容 Windows 反斜杠，与原逻辑保持一致
        let relative = relative.replace('\\', "/");

        // 分割出命名空间 + 资源路径，对应原 path_to_identifier 的解析规则
        let Some((namespace, path)) = relative.split_once('/') else {
            log::warn!("[Recipe] 配方路径缺少命名空间: {}", asset_path);
            continue;
        };

        let identifier = Identifier::new(namespace, path.to_string());
        log::info!("[Recipe] 加载 {}", identifier);
        recipes.push((identifier, recipe));
    }

    if recipes.is_empty() {
        log::info!("[Recipe] 未加载到任何配方定义");
    }

    recipes
}
