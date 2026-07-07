use crate::client::ui::components::{
    SurvivalHotbarPanel, SurvivalInventoryOverlay, SurvivalInventoryRoot, SurvivalItemGrid,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::state::InventoryState;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 生成生存背包 UI 结构
pub fn spawn_survival_inventory_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    commands
        .spawn((
            SurvivalInventoryOverlay,
            Name::new("SurvivalOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    SurvivalInventoryRoot,
                    Name::new("SurvivalRoot"),
                    Node {
                        width: Val::Px(theme.panel_width),
                        height: Val::Px(theme.panel_height),
                        flex_direction: FlexDirection::Column,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_panel),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|root| {
                    // 标题栏
                    root.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(theme.panel_header_h),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect::horizontal(Val::Px(theme.panel_padding)),
                            border: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor::all(theme.border_default),
                    ))
                    .with_children(|header| {
                        header.spawn((
                            Text::new("生存模式背包"),
                            TextFont {
                                font: FontSource::from(ui_font.default.clone()),
                                font_size: FontSize::Px(theme.title_font_size),
                                ..default()
                            },
                            TextColor(theme.text_primary),
                        ));
                    });

                    // 背包网格
                    root.spawn((
                        SurvivalItemGrid,
                        Name::new("SurvivalGrid"),
                        Node {
                            flex_grow: 1.0,
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(9, 1.0),
                            grid_auto_rows: vec![GridTrack::px(theme.slot_size + theme.slot_gap)],
                            column_gap: Val::Px(theme.slot_gap),
                            row_gap: Val::Px(theme.slot_gap),
                            padding: UiRect::all(Val::Px(theme.grid_padding)),
                            ..default()
                        },
                        BackgroundColor(theme.bg_content),
                    ));

                    build_survival_hotbar_panel(root, &theme);
                });
        });
}

/// 生存背包底部的快捷栏
fn build_survival_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    root.spawn((
        SurvivalHotbarPanel,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(theme.creative_hotbar_h),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::top(Val::Px(1.0)),
            padding: UiRect::vertical(Val::Px(6.0)),
            column_gap: Val::Px(4.0),
            ..default()
        },
        BorderColor::all(theme.border_default),
    ));
}

/// 更新生存背包覆盖层可见性
pub fn update_survival_visibility_system(
    state: Res<InventoryState>,
    gamemode: Res<PlayerGameMode>,
    mut query: Query<&mut Visibility, With<SurvivalInventoryOverlay>>,
) {
    let Ok(mut vis) = query.single_mut() else {
        return;
    };
    let target = if state.opened && gamemode.is_survival() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    if *vis != target {
        *vis = target;
    }
}

/// 填充生存背包网格（36 格）
pub fn populate_survival_grid_system(
    grid_query: Query<Entity, With<SurvivalItemGrid>>,
    children_query: Query<&Children>,
    existing_slots: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
) {
    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    // 检查是否已有槽位
    let has_slots = children_query
        .get(grid_entity)
        .map(|children| {
            children
                .iter()
                .any(|child| existing_slots.get(child).is_ok())
        })
        .unwrap_or(false);

    if has_slots {
        return;
    }

    commands.entity(grid_entity).with_children(|grid| {
        for index in 0..36 {
            spawn_empty_slot(grid, SlotKind::SurvivalBackpack, index, &theme);
        }
    });
}

/// 生存背包视觉同步（从 SurvivalInventory 数据同步到 UI）
pub fn survival_grid_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    grid_query: Query<Entity, With<SurvivalItemGrid>>,
    children_query: Query<&Children>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut slot_query: Query<(&InventorySlot, &mut SlotVisual)>,
    mut commands: Commands,
    mut last_snapshot: Local<Option<Vec<(ItemId, u32)>>>,
    mut was_opened: Local<bool>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Ok(grid_entity) = grid_query.single() else {
        return;
    };

    // 背包打开时强制重置缓存（解决 init 系统延迟创建槽位的时序问题）
    if state.opened && !*was_opened {
        *last_snapshot = None;
    }
    *was_opened = state.opened;

    // 构建当前快照（含数量）
    let current: Vec<(ItemId, u32)> = (0..36)
        .map(|i| {
            state
                .survival
                .get_stack(i)
                .map(|s| (s.item.clone(), s.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();

    let force = last_snapshot.is_none();
    let unchanged = !force && (last_snapshot.as_ref() == Some(&current));
    if unchanged {
        return;
    }
    *last_snapshot = Some(current.clone());

    if let Ok(children) = children_query.get(grid_entity) {
        for child in children.iter() {
            if let Ok((slot, mut visual)) = slot_query.get_mut(child) {
                if slot.kind != SlotKind::SurvivalBackpack {
                    continue;
                }
                let (item, count) = current
                    .get(slot.index)
                    .cloned()
                    .unwrap_or((ItemId::air(), 0));
                if force || visual.item != item || visual.count != count {
                    sync_slot_icon(
                        &mut commands,
                        child,
                        &item,
                        count,
                        reg,
                        &children_query,
                        item_registry.as_deref(),
                        item_texture_registry.as_deref(),
                    );
                    visual.item = item;
                    visual.count = count;
                }
            }
        }
    }
}

/// 生存背包底部快捷栏可视同步
pub fn survival_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    mut last_hotbar: Local<Option<Vec<(ItemId, u32)>>>,
    mut last_active: Local<usize>,
    mut was_opened: Local<bool>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };

    let force_reset = state.opened && !*was_opened;
    *was_opened = state.opened;

    if force_reset {
        *last_hotbar = None;
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

    let force = last_hotbar.is_none();
    let changed = force || (last_hotbar.as_ref() != Some(&current));
    if changed {
        *last_hotbar = Some(current.clone());
        for (entity, slot, mut visual) in &mut slot_query {
            if slot.kind != SlotKind::Hotbar {
                continue;
            }
            let (item, count) = current
                .get(slot.index)
                .cloned()
                .unwrap_or((ItemId::air(), 0));
            if force || visual.item != item || visual.count != count {
                sync_slot_icon(
                    &mut commands,
                    entity,
                    &item,
                    count,
                    reg,
                    &children_query,
                    item_registry.as_deref(),
                    item_texture_registry.as_deref(),
                );
                visual.item = item;
                visual.count = count;
            }
        }
    }

    if *last_active != state.hotbar.active_index {
        *last_active = state.hotbar.active_index;
        for (slot, mut border) in &mut border_query {
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
}

/// 生存背包快捷栏初始填充
pub fn init_survival_hotbar_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    hotbar_query: Query<Entity, With<SurvivalHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Ok(panel_entity) = hotbar_query.single() else {
        return;
    };

    let has_hotbar_slots = children_query
        .get(panel_entity)
        .map(|children| {
            children.iter().any(|child| {
                slot_query
                    .get(child)
                    .is_ok_and(|s| s.kind == SlotKind::Hotbar)
            })
        })
        .unwrap_or(false);

    if has_hotbar_slots {
        return;
    }

    commands.entity(panel_entity).with_children(|bar| {
        for (index, item) in state.hotbar.items().iter().enumerate() {
            crate::client::ui::widgets::slot::spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                item,
                reg,
                &theme,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

/// 关闭生存背包时清理快捷栏子实体
pub fn cleanup_survival_hotbar_system(
    state: Res<InventoryState>,
    hotbar_query: Query<Entity, With<SurvivalHotbarPanel>>,
    children_query: Query<&Children>,
    mut commands: Commands,
    mut was_opened: Local<bool>,
) {
    if *was_opened
        && !state.opened
        && let Ok(panel_entity) = hotbar_query.single()
        && let Ok(children) = children_query.get(panel_entity)
    {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }
    *was_opened = state.opened;
}

/// Survival 关闭背包时 cursor 物品放回 (优先来源槽位 → active hotbar → 其余)
pub fn handle_inventory_close(state: &mut InventoryState) {
    use crate::game::inventory::cursor::CursorSource;
    use crate::game::inventory::item::stack::ItemStack;
    if !state.cursor.has_item() {
        return;
    }

    let stack = state.cursor.stack().cloned().unwrap();
    let mut remaining = stack;

    // 1. 优先返回到来源槽位
    if let Some(source) = state.cursor.source {
        match source {
            CursorSource::Hotbar(idx) => {
                remaining = return_to_container(&mut state.hotbar, idx, remaining);
            }
            CursorSource::SurvivalBackpack(idx) => {
                remaining = return_to_container(&mut state.survival, idx, remaining);
            }
            _ => {} // CreativeGrid/Recent/Container: 无特定来源
        }
    }

    if remaining.is_empty() {
        state.cursor.clear();
        return;
    }

    // 2. fallback: 优先放回当前选中的快捷键
    let active = state.hotbar.active_index;
    if let Some(s) = state.hotbar.get_stack(active) {
        if s.item == remaining.item {
            let mut slot_copy = s.clone();
            remaining.merge_from(&mut slot_copy);
            state.hotbar.set_stack(active, slot_copy);
        }
    } else {
        state.hotbar.set_stack(active, remaining);
        remaining = ItemStack::empty();
    }

    // hotbar 其余槽位
    if !remaining.is_empty() {
        for i in 0..state.hotbar.slot_count() {
            if i == active {
                continue;
            }
            if remaining.is_empty() {
                break;
            }
            if let Some(s) = state.hotbar.get_stack_mut(i)
                && s.item == remaining.item
            {
                remaining.merge_from(s);
            }
        }
    }
    if !remaining.is_empty() {
        for i in 0..state.hotbar.slot_count() {
            if i == active {
                continue;
            }
            if state.hotbar.get_stack(i).is_none() {
                state.hotbar.set_stack(i, remaining);
                remaining = ItemStack::empty();
                break;
            }
        }
    }

    // backpack
    if !remaining.is_empty() {
        for i in 0..36 {
            if remaining.is_empty() {
                break;
            }
            if let Some(s) = state.survival.get_stack_mut(i)
                && s.item == remaining.item
            {
                remaining.merge_from(s);
            }
        }
    }
    if !remaining.is_empty() {
        for i in 0..36 {
            if state.survival.get_stack(i).is_none() {
                state.survival.set_stack(i, remaining);
                remaining = ItemStack::empty();
                break;
            }
        }
    }
    state.cursor.clear();
    if !remaining.is_empty() {
        log::warn!("[Survival] backpack full, lost: {:?}", remaining);
    }
}

/// 尝试将物品返回到指定容器的槽位（先合并同种、再放入空位）
fn return_to_container<C: crate::game::inventory::container::InventoryContainer>(
    container: &mut C,
    index: usize,
    mut remaining: crate::game::inventory::item::stack::ItemStack,
) -> crate::game::inventory::item::stack::ItemStack {
    if remaining.is_empty() {
        return remaining;
    }
    // 先尝试合并到该槽位
    if let Some(s) = container.get_stack_mut(index)
        && s.item == remaining.item
    {
        remaining.merge_from(s);
    }
    // 如果槽位为空，直接放入
    if !remaining.is_empty() && container.get_stack(index).is_none() {
        container.set_stack(index, remaining);
        remaining = crate::game::inventory::item::stack::ItemStack::empty();
    }
    remaining
}
