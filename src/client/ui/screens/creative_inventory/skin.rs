//! 填充创造物品栏中依赖渲染资源的延迟视觉元素。

use bevy::prelude::*;

use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{CreativeTabIcon, CreativeTitleIcon};
use crate::client::ui::resources::creative_assets::{
    CreativeUiAssets, SLOT_SLICE, sliced_image_node,
};
use crate::client::ui::widgets::slot::resolve_item_image_node;
use crate::client::ui::widgets::tab::category_icon_item_path;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::state::LocalInventory;
use crate::shared::identifier::Identifier;
use crate::shared::item_id::ItemId;

/// 标题图标使用的代表方块：草方块，与设计稿标题一致。
const CREATIVE_TITLE_ICON_ITEM: &str = "grass";

/// 为通用生成函数创建的槽位实体附加九宫格物品框皮肤。
///
/// 皮肤与生成指令同帧写入，避免延迟一帧导致的底色闪烁；
/// 纯色 `BackgroundColor` 被移除防止衬在纹理下方。
/// `keep_border` 为真时保留边框组件，供快捷栏显示选中高亮。
pub(super) fn attach_creative_slot_skin(
    parent: &mut ChildSpawnerCommands,
    slot: Entity,
    assets: &CreativeUiAssets,
    keep_border: bool,
) {
    let mut commands = parent.commands();
    let mut entity = commands.entity(slot);
    entity.remove::<BackgroundColor>();
    if !keep_border {
        entity.remove::<BorderColor>();
    }
    entity.insert(sliced_image_node(assets.slot.clone(), SLOT_SLICE));
}

/// 为标题图标节点填充草方块 GUI 图标。
///
/// Startup 阶段 3D 图标烘焙尚未完成，因此以“无 ImageNode 即待填充”
/// 作为触发条件，资源就绪后自动补齐且只执行一次。
pub fn apply_creative_title_icon_system(
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
    query: Query<(Entity, &CreativeTitleIcon), Without<ImageNode>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };
    let Ok((entity, _)) = query.single() else {
        return;
    };

    let item = ItemId::new(Identifier::new("century_journey", CREATIVE_TITLE_ICON_ITEM));
    if let Some(image_node) = resolve_item_image_node(
        &item,
        reg,
        render_assets,
        &gui_item_icons,
        item_registry.as_deref(),
        item_texture_registry.as_deref(),
    ) {
        commands.entity(entity).insert(image_node);
    }
}

/// 为分类标签图标节点填充代表物品 GUI 图标。
///
/// 图标物品在标签生成时决定；分类数据变化会重建标签实体，
/// 新实体缺少 ImageNode 时本系统再次补齐。
#[allow(clippy::too_many_arguments)]
pub fn apply_creative_tab_icon_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut commands: Commands,
    query: Query<(Entity, &CreativeTabIcon), Without<ImageNode>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };

    for (entity, icon) in &query {
        let Some(category) = state.creative.categories.get(icon.category_index) else {
            continue;
        };
        let Some(path) = category_icon_item_path(category) else {
            continue;
        };
        let item = ItemId::new(Identifier::new("century_journey", path));
        if let Some(image_node) = resolve_item_image_node(
            &item,
            reg,
            render_assets,
            &gui_item_icons,
            item_registry.as_deref(),
            item_texture_registry.as_deref(),
        ) {
            commands.entity(entity).insert(image_node);
        }
    }
}
