use bevy::prelude::*;

use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::tag::compiler::TagRegistryCompiler;
use crate::content::tag::loader::load_tag_actions;
use crate::engine::asset::manager::AssetManager;
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
                .run_if(crate::app::flow::fresh_game_session),
        );
    }
}

pub(crate) fn init_tag_registry_system(
    mut commands: Commands,
    asset: Res<AssetManager>,
    block_registry: Res<BlockRegistry>,
    item_registry: Option<Res<ItemRegistry>>,
) {
    // 1. 创建 Compiler
    let mut compiler = TagRegistryCompiler::new();

    // 2. 收集 default_tags (BlockProperty.tags)
    compiler.collect_from_blocks(&block_registry);

    // 3. 收集 default_tags (ItemDefinition.tags)
    if let Some(ref ir) = item_registry {
        compiler.collect_from_items(ir);
    }

    // 4. 加载 TagActions 并应用
    let actions = load_tag_actions(&asset);
    let mut applied = 0usize;
    for (tag_id, action) in actions {
        compiler.apply_action(tag_id, &action);
        applied += 1;
    }
    if applied > 0 {
        log::info!("[标签系统] 已加载 {} 个标签定义", applied);
    }

    // 5. 展开 Tag 引用
    compiler.resolve_references();

    // 6. 检测未解析引用
    let unresolved = compiler.detect_unresolved();
    if !unresolved.is_empty() {
        log::warn!("[标签系统] {} 个标签引用未解析", unresolved.len());
        for (from, to) in &unresolved {
            log::warn!("  {} → {} (目标不存在)", from.to_full(), to.to_full());
        }
    }

    // 7. 构建 Runtime
    let (block_runtime, item_index) = compiler.build_runtime(&block_registry);

    log::info!(
        "[标签系统] 编译完成: {} 个方块标签, {} 个物品标签",
        block_runtime.total_tags(),
        item_index.total_tags()
    );

    // 8. 插入 Resource
    commands.insert_resource(block_runtime);
    commands.insert_resource(item_index);
}
