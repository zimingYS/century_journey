//! 配方加载与注册系统。

use bevy::prelude::*;

use crate::content::recipe::registry::RecipeRegistry;
use crate::content::validation::ContentCompilation;

/// 从内容编译结果把配方注册到运行时索引。
pub(super) fn load_recipes_system(
    mut registry: ResMut<RecipeRegistry>,
    compilation: Res<ContentCompilation>,
) {
    let recipes = compilation.content.recipes.clone();

    for (id, recipe) in recipes {
        registry.register(id, recipe);
    }

    log::info!("[配方] 已加载 {} 个配方", registry.all_recipes().count());
}
