use bevy::prelude::*;
use crate::engine::constant::world::CHUNK_SIZE;
use crate::game::inventory::item::icon::IconDefinition;
use crate::game::inventory::item::id::ItemId;
use crate::game::inventory::item::registry::ItemRegistry;
use crate::game::inventory::item::texture_registry::ItemTextureRegistry;
use crate::game::inventory::slot::SlotAction;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;

/// 槽位
#[derive(Component, Debug, Clone, Copy)]
pub struct InventorySlot {
    pub kind: SlotKind,
    pub index: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SlotKind {
    Hotbar,
    CreativeGrid,
    Recent,
    SurvivalBackpack,
    Container,
}

/// 槽位图标子实体标记
#[derive(Component)]
pub struct SlotIcon;

/// 槽位数量文本子实体标记
#[derive(Component)]
pub struct SlotCountText;

/// 槽位的视觉状态缓存
#[derive(Component, Debug, Clone)]
pub struct SlotVisual {
    pub item: ItemId,
    pub count: u32,
}

/// 默认槽位为空气
impl Default for SlotVisual {
    fn default() -> Self {
        Self { item: ItemId::air(), count: 0 }
    }
}

/// 分类标签按钮
#[derive(Component, Debug, Clone, Copy)]
pub struct CategoryTab {
    pub category_index: usize,
}

/// 搜索框标记
#[derive(Component)]
pub struct CreativeSearchInput;

/// 搜索状态
#[derive(Resource, Default)]
pub struct SearchInputState {
    pub active: bool,
}

#[deprecated(note = "请使用 SlotInteractionEvent")]
pub type SlotClickedEvent = SlotInteractionEvent;

/// 槽位点击事件
#[derive(Message, Debug)]
pub struct SlotInteractionEvent {
    pub kind: SlotKind,
    pub index: usize,
    pub action: SlotAction,
}

/// 分类切换事件
#[derive(Message, Debug)]
pub struct CategoryClickedEvent {
    pub category_index: usize,
}

fn layer_to_atlas_index(layer_idx: u32) -> usize {
    (layer_idx as usize) * CHUNK_SIZE * CHUNK_SIZE
}

/// 生成空槽位（用于HUD快捷栏初始化）
pub fn spawn_empty_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
) {
    parent.spawn((
        InventorySlot { kind, index },
        SlotVisual { item: ItemId::air(), count: 0 },
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
    )    ).with_children(|slot| {
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
    parent.spawn((
        InventorySlot { kind, index },
        SlotVisual { item: item.clone(), count: 0 },
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
    )).with_children(|slot| {
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
    let icon_def = if let Some(block_id) = item.as_block_id() {
        // Block 变体: 直接使用 block_id 作为图标
        Some(IconDefinition::block(block_id))
    } else if let Some(reg) = item_registry {
        reg.get(item).map(|def| def.icon.clone())
    } else {
        None
    };

    let Some(icon) = icon_def else {
        // 无图标: 隐藏占位
        parent.spawn((
            SlotIcon,
            Node { width: Val::Percent(80.0), height: Val::Percent(80.0), ..default() },
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
                    Node { width: Val::Percent(80.0), height: Val::Percent(80.0), ..default() },
                    Visibility::Hidden,
                ));
                return;
            };

            parent.spawn((
                SlotIcon,
                ImageNode {
                    image: block_registry.base_texture.clone(),
                    texture_atlas: Some(TextureAtlas {
                        layout: block_registry.atlas_layout.clone(),
                        index: atlas_idx,
                    }),
                    ..default()
                },
                Node { width: Val::Percent(80.0), height: Val::Percent(80.0), ..default() },
            ));
        }

        // 独立纹理图标
        IconDefinition::Texture(path) => {
            let handle = item_texture_registry
                .and_then(|reg| reg.get(&path).cloned())
                .unwrap_or_default();

            parent.spawn((
                SlotIcon,
                ImageNode {
                    image: handle,
                    texture_atlas: None,
                    ..default()
                },
                Node { width: Val::Percent(80.0), height: Val::Percent(80.0), ..default() },
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
    let Ok(children) = children_query.get(slot_entity) else { return };

    // ── 更新图标 ──
    if let Some(&icon_entity) = children.first() {
        if item.is_air() {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        } else {
            let icon_def = if let Some(block_id) = item.as_block_id() {
                Some(IconDefinition::block(block_id))
            } else if let Some(reg) = item_registry {
                reg.get(item).map(|def| def.icon.clone())
            } else {
                None
            };

            if let Some(icon) = icon_def {
                match icon {
                    IconDefinition::Block(id) => {
                        if let Some(atlas_idx) = block_registry.get_icon_atlas_index(&id) {
                            commands.entity(icon_entity).insert((
                                Visibility::Inherited,
                                ImageNode {
                                    image: block_registry.base_texture.clone(),
                                    texture_atlas: Some(TextureAtlas {
                                        layout: block_registry.atlas_layout.clone(),
                                        index: atlas_idx,
                                    }),
                                    ..default()
                                },
                            ));
                        }
                    }
                    IconDefinition::Texture(path) => {
                        let handle = item_texture_registry
                            .and_then(|reg| reg.get(&path).cloned())
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
            commands.entity(count_entity).insert((
                Visibility::Inherited,
                Text::new(count.to_string()),
            ));
        } else {
            commands.entity(count_entity).insert(Visibility::Hidden);
        }
    }
}