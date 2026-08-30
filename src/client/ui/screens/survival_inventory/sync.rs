//! 将权威物品栏和玩家状态投影到生存物品栏界面。

use bevy::prelude::*;

use super::layout::{
    BACKPACK_SLOT_SIZE, HOTBAR_SELECTION_HEIGHT, HOTBAR_SELECTION_OFFSET, HOTBAR_SELECTION_WIDTH,
    HOTBAR_SLOT_HEIGHT, HOTBAR_SLOT_WIDTH, slot_theme,
};
use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{
    SurvivalDefenseText, SurvivalHealthText, SurvivalHotbarPanel, SurvivalHotbarSelectionFrame,
    SurvivalHungerText, SurvivalItemGrid, SurvivalPlayerPreviewCamera,
};
use crate::client::ui::resources::survival_assets::SurvivalUiAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::engine::localization::Localization;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::state::{AccessorySlotDefinitions, LocalInventory, LocalInventoryMut};
use crate::game::player::identity::LocalPlayer;
use crate::game::player::survival::health::Health;
use crate::game::player::survival::hunger::Hunger;
use crate::game::player::survival::protection::Defense;
use crate::shared::item_id::ItemId;
/// 根据当前界面栈启停离屏预览相机。
pub fn update_survival_visibility_system(
    stack: Res<crate::client::ui::navigation::UiScreenStack>,
    gamemode: Res<PlayerGameMode>,
    mut camera_query: Query<&mut Camera, With<SurvivalPlayerPreviewCamera>>,
) {
    let visible = stack.contains(crate::client::ui::navigation::UiScreen::Inventory)
        && gamemode.is_survival();
    if let Ok(mut camera) = camera_query.single_mut() {
        camera.is_active = visible;
    }
}

/// 存档恢复发生在 Startup 之后，因此每帧只做廉价的扩容检查。
pub fn sync_accessory_slot_count_system(
    definitions: Res<AccessorySlotDefinitions>,
    mut state: LocalInventoryMut,
) {
    state
        .survival
        .ensure_accessory_slots(definitions.slots.len());
}

/// 首次创建生存背包的逻辑槽位视图。
pub fn populate_survival_grid_system(
    grid_query: Query<Entity, With<SurvivalItemGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    let Ok(grid_entity) = grid_query.single() else {
        return;
    };
    let has_slots = children_query.get(grid_entity).is_ok_and(|children| {
        children
            .iter()
            .any(|child| existing_slots.get(child).is_ok())
    });
    if has_slots {
        return;
    }

    let slot_theme = slot_theme(&theme, BACKPACK_SLOT_SIZE);
    commands.entity(grid_entity).with_children(|grid| {
        for index in 0..SurvivalInventory::BACKPACK_SIZE {
            spawn_empty_slot(
                grid,
                SlotKind::SurvivalBackpack,
                index,
                &slot_theme,
                &ui_font,
            );
        }
    });
}

/// 在物品或模型资源变化时同步生存背包图标。
/// 背包网格同步显式读取全部渲染缓存，并以本地快照控制重建。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn survival_grid_visual_sync_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    children_query: Query<&Children>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    mut commands: Commands,
    mut last_snapshot: Local<Option<(Vec<(ItemId, u32)>, u64)>>,
    mut was_opened: Local<bool>,
) {
    let (Some(registry), Some(render_assets)) =
        (block_registry.as_deref(), block_render_assets.as_deref())
    else {
        return;
    };

    if state.opened && !*was_opened {
        *last_snapshot = None;
    }
    *was_opened = state.opened;

    let current: Vec<(ItemId, u32)> = (0..state.survival.slot_count())
        .map(|index| {
            state
                .survival
                .get_stack(index)
                .map(|stack| (stack.item.clone(), stack.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();
    let revision = gui_item_icons.revision();
    if last_snapshot
        .as_ref()
        .is_some_and(|(snapshot, cached_revision)| {
            snapshot == &current && *cached_revision == revision
        })
    {
        return;
    }
    let force = last_snapshot.is_none();
    let revision_changed = last_snapshot
        .as_ref()
        .is_some_and(|(_, cached_revision)| *cached_revision != revision);
    *last_snapshot = Some((current.clone(), revision));

    for (entity, slot, mut visual) in &mut slot_query {
        let Some(index) =
            crate::game::inventory::interaction::routing::survival_index(slot.kind, slot.index)
        else {
            continue;
        };
        let (item, count) = current.get(index).cloned().unwrap_or((ItemId::air(), 0));
        if force || revision_changed || visual.item != item || visual.count != count {
            sync_slot_icon(
                &mut commands,
                entity,
                &item,
                count,
                registry,
                render_assets,
                &gui_item_icons,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
            visual.item = item;
            visual.count = count;
        }
    }
}

/// 同步生存物品栏内的快捷栏图标和选中边框。
/// 生存快捷栏同步与 HUD 使用相同缓存，但拥有独立的失效快照。
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn survival_hotbar_visual_sync_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut selection_query: Query<(&SurvivalHotbarSelectionFrame, &mut Visibility)>,
    mut last_hotbar: Local<Option<(Vec<(ItemId, u32)>, u64)>>,
    mut last_active: Local<Option<usize>>,
    mut was_opened: Local<bool>,
) {
    let (Some(registry), Some(render_assets)) =
        (block_registry.as_deref(), block_render_assets.as_deref())
    else {
        return;
    };
    if state.opened && !*was_opened {
        *last_hotbar = None;
        *last_active = None;
    }
    *was_opened = state.opened;

    let current: Vec<(ItemId, u32)> = (0..HOTBAR_SIZE)
        .map(|index| {
            state
                .hotbar
                .get_stack(index)
                .map(|stack| (stack.item.clone(), stack.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();
    let revision = gui_item_icons.revision();
    let changed = last_hotbar
        .as_ref()
        .is_none_or(|(snapshot, cached_revision)| {
            snapshot != &current || *cached_revision != revision
        });
    if changed {
        let force = last_hotbar.is_none();
        let revision_changed = last_hotbar
            .as_ref()
            .is_some_and(|(_, cached_revision)| *cached_revision != revision);
        *last_hotbar = Some((current.clone(), revision));
        for (entity, slot, mut visual) in &mut slot_query {
            if slot.kind != SlotKind::Hotbar {
                continue;
            }
            let (item, count) = current
                .get(slot.index)
                .cloned()
                .unwrap_or((ItemId::air(), 0));
            if force || revision_changed || visual.item != item || visual.count != count {
                sync_slot_icon(
                    &mut commands,
                    entity,
                    &item,
                    count,
                    registry,
                    render_assets,
                    &gui_item_icons,
                    &children_query,
                    item_registry.as_deref(),
                    item_texture_registry.as_deref(),
                );
                visual.item = item;
                visual.count = count;
            }
        }
    }

    if *last_active != Some(state.hotbar.active_index) {
        *last_active = Some(state.hotbar.active_index);
        for (slot, mut border) in &mut border_query {
            if slot.kind == SlotKind::Hotbar {
                *border = BorderColor::all(if slot.index == state.hotbar.active_index {
                    theme.border_selected
                } else {
                    theme.border_default
                });
            }
        }
        // 生存面板快捷栏使用金色选中框图片，HUD 快捷栏继续使用边框。
        // Inherited 而非 Visible：Visible 会无视父级强制显示，
        // 导致物品栏关闭后选中框残留在屏幕上；Inherited 跟随面板可见性。
        for (frame, mut visibility) in &mut selection_query {
            *visibility = if frame.index == state.hotbar.active_index {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

/// 为生存物品栏面板补建快捷栏槽位。
/// 首次打开生存背包时创建快捷栏槽位，内容资源缺失时安全延后。
#[allow(clippy::too_many_arguments)]
pub fn init_survival_hotbar_system(
    state: LocalInventory,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    hotbar_query: Query<Entity, With<SurvivalHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    survival_assets: Res<SurvivalUiAssets>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
) {
    let (Some(registry), Some(render_assets)) =
        (block_registry.as_deref(), block_render_assets.as_deref())
    else {
        return;
    };
    let Ok(panel_entity) = hotbar_query.single() else {
        return;
    };
    let has_slots = children_query.get(panel_entity).is_ok_and(|children| {
        children.iter().any(|child| {
            slot_query
                .get(child)
                .is_ok_and(|slot| slot.kind == SlotKind::Hotbar)
        })
    });
    if has_slots {
        return;
    }

    let mut hotbar_theme = slot_theme(&theme, HOTBAR_SLOT_WIDTH);
    hotbar_theme.slot_height = HOTBAR_SLOT_HEIGHT;
    // with_children 闭包内不能再次借用 commands，先收集槽位实体再补挂选中框。
    let mut spawned_slots: Vec<(usize, Entity)> = Vec::with_capacity(state.hotbar.stacks.len());
    commands.entity(panel_entity).with_children(|bar| {
        for (index, stack) in state.hotbar.stacks.iter().enumerate() {
            let item = stack
                .as_ref()
                .map_or_else(ItemId::air, |stack| stack.item.clone());
            let slot_entity = crate::client::ui::widgets::slot::spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                &item,
                registry,
                render_assets,
                &gui_item_icons,
                &hotbar_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
            spawned_slots.push((index, slot_entity));
        }
    });
    // 金色选中框叠加在槽位外沿（素材含 15px 外扩边）。
    for (index, slot_entity) in spawned_slots {
        commands.entity(slot_entity).with_children(|slot| {
            slot.spawn((
                SurvivalHotbarSelectionFrame { index },
                ImageNode {
                    image: survival_assets.hotbar_selection.clone(),
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(HOTBAR_SELECTION_OFFSET.x),
                    top: Val::Px(HOTBAR_SELECTION_OFFSET.y),
                    width: Val::Px(HOTBAR_SELECTION_WIDTH),
                    height: Val::Px(HOTBAR_SELECTION_HEIGHT),
                    ..default()
                },
                Visibility::Hidden,
            ));
        });
    }
}

/// 把权威生命、饥饿和防御数值投影到界面文本。
/// 单次查询同步三类生存文本，组件选项明确文本节点的实际角色。
#[allow(clippy::type_complexity)]
pub fn survival_stats_visual_sync_system(
    player_query: Query<(&Health, &Hunger, Option<&Defense>), With<LocalPlayer>>,
    localization: Res<Localization>,
    mut text_query: Query<(
        &mut Text,
        Option<&SurvivalHealthText>,
        Option<&SurvivalDefenseText>,
        Option<&SurvivalHungerText>,
    )>,
) {
    let Ok((health, hunger, defense)) = player_query.single() else {
        return;
    };
    for (mut text, health_marker, defense_marker, hunger_marker) in &mut text_query {
        if health_marker.is_some() {
            *text = Text::new(localization.format(
                "survival.health",
                &[
                    ("current", &format!("{:.0}", health.current)),
                    ("max", &format!("{:.0}", health.max)),
                ],
            ));
        } else if defense_marker.is_some() {
            *text = Text::new(localization.format(
                "survival.defense",
                &[(
                    "value",
                    &format!("{:.0}", defense.map_or(0.0, |value| value.0)),
                )],
            ));
        } else if hunger_marker.is_some() {
            *text = Text::new(localization.format(
                "survival.hunger",
                &[
                    ("current", &format!("{:.0}", hunger.current)),
                    ("max", &format!("{:.0}", hunger.max)),
                ],
            ));
        }
    }
}
