//! 组装配方加载和注册表刷新流程。

use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::recipe::systems::load_recipes_system;
use crate::shared::states::app_state::AppState;
use bevy::prelude::*;

/// Content 层 Recipe 插件。
/// 完全对照 ItemContentPlugin 的结构。
pub struct RecipeContentPlugin;

impl Plugin for RecipeContentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecipeRegistry>().add_systems(
            OnEnter(AppState::InGame),
            load_recipes_system
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}
