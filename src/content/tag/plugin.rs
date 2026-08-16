//! 组装标签加载、编译和注册表发布流程。

use bevy::prelude::*;

use crate::content::lifecycle::{ContentReloadSet, content_reload_requested};
use crate::content::tag::systems::init_tag_registry_system;
use crate::shared::states::app_state::AppState;

/// Content 层 Tag Plugin。
///
/// 使用 V3 Compiler 架构：
///   Definition → Compiler → RuntimeTagRegistry
///
/// Compiler 在编译完成后立即释放，不进入 Runtime。
pub struct TagContentPlugin;

impl Plugin for TagContentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(AppState::InGame),
            init_tag_registry_system
                .after(crate::content::item::model::load_item_models_system)
                .in_set(ContentReloadSet::Load)
                .run_if(content_reload_requested),
        );
    }
}
