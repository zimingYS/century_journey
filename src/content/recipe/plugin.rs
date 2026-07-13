use crate::content::recipe::loader::load_recipe_definitions;
use crate::content::recipe::registry::RecipeRegistry;
use crate::engine::asset::manager::AssetManager;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// Content 层 Recipe 插件。
/// 完全对照 ItemContentPlugin 的结构。
pub struct RecipeContentPlugin;

impl Plugin for RecipeContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecipeRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_recipes_system.run_if(crate::app::flow::fresh_game_session),
        );
    }
}

fn load_recipes_system(mut registry: ResMut<RecipeRegistry>, asset: Res<AssetManager>) {
    let recipes = load_recipe_definitions(&asset);

    for (id, recipe) in recipes {
        registry.register(id, recipe);
    }

    log::info!("[Recipe] 已加载 {} 个配方", registry.all_recipes().count());
}
