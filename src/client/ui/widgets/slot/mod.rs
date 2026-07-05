pub mod components;

pub use crate::shared::ui_types::{SearchInputState, SlotKind};
pub use components::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SlotCountText, SlotIcon,
    SlotInteractionEvent, SlotVisual,
};

use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 统一解析物品图标定义。
/// 优先查询 BlockRegistry 获取方块别名，再回退到 ItemRegistry 的 icon 字段。
pub fn resolve_item_icon(
    item: &ItemId,
    item_registry: Option<&ItemRegistry>,
) -> Option<IconDefinition> {
    let reg = item_registry?;
    if let Some(block_id) = reg.block_identifier(item) {
        Some(IconDefinition::block(block_id.to_string()))
    } else {
        reg.get(item).map(|def| def.icon.clone())
    }
}

/// 生成空槽位（用于HUD快捷栏初始化）
pub fn spawn_empty_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            slot.spawn((
                SlotIcon,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
        });
}

/// 生成带物品的槽位（用于创造网格/最近使用/快捷栏）
pub fn spawn_slot_with_item(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    item: &ItemId,
    registry: &BlockRegistry,
    theme: &UiTheme,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: item.clone(),
                count: 0,
            },
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            spawn_icon_child(slot, item, registry, item_registry, item_texture_registry);
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
        });
}

/// 生成槽位的图标子节点
pub fn spawn_icon_child(
    parent: &mut ChildSpawnerCommands,
    item: &ItemId,
    block_registry: &BlockRegistry,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    // 确定纹理标识符
    let icon_def = resolve_item_icon(item, item_registry);

    let Some(icon) = icon_def else {
        // 无图标: 隐藏占位
        parent.spawn((
            SlotIcon,
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(80.0),
                ..default()
            },
            Visibility::Hidden,
        ));
        return;
    };

    match icon {
        // 方块图标
        IconDefinition::Block(id) => {
            let Some(atlas_idx) = block_registry.get_icon_atlas_index(&id) else {
                parent.spawn((
                    SlotIcon,
                    Node {
                        width: Val::Percent(80.0),
                        height: Val::Percent(80.0),
                        ..default()
                    },
                    Visibility::Hidden,
                ));
                return;
            };

            parent.spawn((
                SlotIcon,
                ImageNode {
                    image: block_registry.base_texture().clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: block_registry.atlas_layout().clone(),
                        index: atlas_idx,
                    }),
                    ..default()
                },
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
            ));
        }

        // 独立纹理图标
        IconDefinition::Texture(path) => {
            let handle = item_texture_registry
                .and_then(|reg| reg.get_handle(&path).cloned())
                .unwrap_or_default();

            parent.spawn((
                SlotIcon,
                ImageNode {
                    image: handle,
                    texture_atlas: None,
                    ..default()
                },
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
            ));
        }
    }
}

/// 原地更新槽位图标 + 数量，不销毁槽位实体
pub fn sync_slot_icon(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    count: u32,
    block_registry: &BlockRegistry,
    children_query: &Query<&Children>,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    let Ok(children) = children_query.get(slot_entity) else {
        return;
    };

    // ── 更新图标 ──
    if let Some(&icon_entity) = children.first() {
        if item.is_air() {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        } else {
            let icon_def = resolve_item_icon(item, item_registry);

            if let Some(icon) = icon_def {
                match icon {
                    IconDefinition::Block(id) => {
                        if let Some(atlas_idx) = block_registry.get_icon_atlas_index(&id) {
                            commands.entity(icon_entity).insert((
                                Visibility::Inherited,
                                ImageNode {
                                    image: block_registry.base_texture().clone(),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: block_registry.atlas_layout().clone(),
                                        index: atlas_idx,
                                    }),
                                    ..default()
                                },
                            ));
                        }
                    }
                    IconDefinition::Texture(path) => {
                        let handle = item_texture_registry
                            .and_then(|reg| reg.get_handle(&path).cloned())
                            .unwrap_or_default();
                        commands.entity(icon_entity).insert((
                            Visibility::Inherited,
                            ImageNode {
                                image: handle,
                                texture_atlas: None,
                                ..default()
                            },
                        ));
                    }
                }
            }
        }
    }

    // 数量文本
    if let Some(&count_entity) = children.get(1) {
        if count > 1 {
            commands
                .entity(count_entity)
                .insert((Visibility::Inherited, Text::new(count.to_string())));
        } else {
            commands.entity(count_entity).insert(Visibility::Hidden);
        }
    }
}

/// 通用快捷栏视觉同步。
/// 供 creative/survival/HUD 的 hotbar sync 系统复用。
pub fn sync_hotbar_panel_visuals(
    state: &crate::game::inventory::state::InventoryState,
    reg: &BlockRegistry,
    panel_entity: Entity,
    children_query: &Query<&Children>,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
    slot_query: &mut Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    border_query: &mut Query<(&InventorySlot, &mut BorderColor)>,
    theme: &UiTheme,
    commands: &mut Commands,
    last_snapshot: &mut Option<Vec<(crate::shared::item_id::ItemId, u32)>>,
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

    let force = last_snapshot.is_none();
    if !force && last_snapshot.as_ref().map_or(false, |old| old == &current) {
        return;
    }
    *last_snapshot = Some(current.clone());

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
                if force || visual.item != item || visual.count != count {
                    sync_slot_icon(
                        commands,
                        entity,
                        &item,
                        count,
                        reg,
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

/// 生成纯展示槽位（用于常驰HUD，不响应鼠标交互）
pub fn spawn_display_only_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
            // 不再附加 Button / Pickable
            Node {
                width: Val::Px(theme.slot_size),
                height: Val::Px(theme.slot_size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(theme.slot_border)),
                ..default()
            },
            BackgroundColor(theme.bg_slot),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|slot| {
            slot.spawn((
                SlotIcon,
                Node {
                    width: Val::Percent(80.0),
                    height: Val::Percent(80.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(1.0),
                    right: Val::Px(3.0),
                    ..default()
                },
                Visibility::Hidden,
            ));
        });
}
