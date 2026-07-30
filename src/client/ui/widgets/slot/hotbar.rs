//! 同步背包界面内快捷栏的图标、数量和选中边框。

use bevy::prelude::*;

use super::components::{InventorySlot, SlotVisual};
use super::icon::sync_slot_icon;
use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::slot::SlotKind;
/// 同步快捷栏面板的槽位图标、数量和选中边框。
/// 槽位面板复用调用方借用的渲染缓存，避免建立拥有全局资源的辅助对象。
#[allow(clippy::too_many_arguments)]
pub fn sync_hotbar_panel_visuals(
    state: &crate::game::inventory::state::InventoryState,
    reg: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    gui_item_icons: &GuiItemIconCache,
    panel_entity: Entity,
    children_query: &Query<&Children>,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
    slot_query: &mut Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    border_query: &mut Query<(&InventorySlot, &mut BorderColor)>,
    theme: &UiTheme,
    commands: &mut Commands,
    last_snapshot: &mut Option<(Vec<(crate::shared::item_id::ItemId, u32)>, u64)>,
    force_reset: bool,
) {
    use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
    use crate::shared::item_id::ItemId;

    if force_reset {
        *last_snapshot = None;
    }

    let current: Vec<(ItemId, u32)> = (0..HOTBAR_SIZE)
        .map(|i| {
            state
                .hotbar
                .get_stack(i)
                .map(|s| (s.item.clone(), s.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();

    let revision = gui_item_icons.revision();
    let force = last_snapshot.is_none();
    let revision_changed = last_snapshot
        .as_ref()
        .is_some_and(|(_, cached_revision)| *cached_revision != revision);
    let unchanged = !force
        && last_snapshot
            .as_ref()
            .is_some_and(|(items, cached_revision)| {
                items == &current && *cached_revision == revision
            });
    if unchanged {
        return;
    }
    *last_snapshot = Some((current.clone(), revision));

    if let Ok(children) = children_query.get(panel_entity) {
        for child in children.iter() {
            if let Ok((entity, slot, mut visual)) = slot_query.get_mut(child) {
                if slot.kind != SlotKind::Hotbar {
                    continue;
                }
                let (item, count) = current
                    .get(slot.index)
                    .cloned()
                    .unwrap_or((ItemId::air(), 0));
                if force || revision_changed || visual.item != item || visual.count != count {
                    sync_slot_icon(
                        commands,
                        entity,
                        &item,
                        count,
                        reg,
                        render_assets,
                        gui_item_icons,
                        children_query,
                        item_registry,
                        item_texture_registry,
                    );
                    visual.item = item;
                    visual.count = count;
                }
            }
        }
    }

    for (slot, mut border) in border_query.iter_mut() {
        if slot.kind != SlotKind::Hotbar {
            continue;
        }
        *border = BorderColor::all(if slot.index == state.hotbar.active_index {
            theme.border_selected
        } else {
            theme.border_default
        });
    }
}
