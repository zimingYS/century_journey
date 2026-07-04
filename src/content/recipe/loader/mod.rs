use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::engine::asset::identifier::asset_id;
use crate::engine::asset::manager::AssetManager;
use crate::shared::identifier::Identifier;
use std::path::PathBuf;

/// 从 assets/definitions/recipes 加载所有 RecipeDefinition
pub fn load_recipe_definitions(asset: &AssetManager) -> Vec<(Identifier, RecipeDefinition)> {
    let root = PathBuf::from("assets/definitions/recipes");

    if !root.exists() {
        log::info!("[Recipe] 配方目录不存在，跳过加载：{:?}", root);
        return Vec::new();
    }

    let mut recipes = Vec::new();

    collect_recipe_files(asset, &root, &mut recipes);

    recipes
}

fn collect_recipe_files(
    asset: &AssetManager,
    dir: &std::path::Path,
    recipes: &mut Vec<(Identifier, RecipeDefinition)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect_recipe_files(asset, &path, recipes);
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let Some(identifier) = path_to_identifier(&path) else {
            continue;
        };

        let asset_path = format!(
            "definitions/recipes/{}",
            path.strip_prefix("assets/definitions/recipes/")
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/")
        );

        let asset_id = asset_id(&asset_path);

        match asset.read_json_sync::<RecipeDefinition>(&asset_id) {
            Ok(recipe) => {
                log::info!("[Recipe] 加载 {}", identifier);
                recipes.push((identifier, recipe));
            }
            Err(err) => {
                log::error!("[Recipe] 加载失败 {:?}: {}", path, err);
            }
        }
    }
}

fn path_to_identifier(path: &std::path::Path) -> Option<Identifier> {
    let components: Vec<&str> = path
        .iter()
        .rev()
        .take_while(|c| *c != "recipes")
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .filter_map(|c| c.to_str())
        .collect();

    if components.len() < 2 {
        return None;
    }

    let namespace = components[0];

    let stem = path.file_stem()?.to_str()?;

    let mut path_parts = components[1..components.len() - 1].to_vec();
    path_parts.push(stem);

    Some(Identifier::new(namespace, path_parts.join("/")))
}
