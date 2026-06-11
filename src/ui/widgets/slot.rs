use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::inventory::item::id::ItemId;
use crate::ui::theme::ui_theme::UiTheme;
use crate::voxel::registry::BlockRegistry;

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

/// 槽位的视觉状态缓存
#[derive(Component, Debug, Clone)]
pub struct SlotVisual {
    pub item: ItemId,
}

/// 默认槽位为空气
impl Default for SlotVisual {
    fn default() -> Self {
        Self { item: ItemId::air() }
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

/// 槽位点击事件
#[derive(Message, Debug)]
pub struct SlotClickedEvent {
    pub kind: SlotKind,
    pub index: usize,
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
        SlotVisual { item: ItemId::air() },
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
        slot.spawn((
            SlotIcon,
            Node {
                width: Val::Percent(80.0),
                height: Val::Percent(80.0),
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
) {
    parent.spawn((
        InventorySlot { kind, index },
        SlotVisual { item: item.clone() },
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
        spawn_icon_child(slot, item, registry);
    });
}

/// 生成槽位的图标子节点
pub fn spawn_icon_child(
    parent: &mut ChildSpawnerCommands,
    item: &ItemId,
    registry: &BlockRegistry,
) {
    if item.is_air() {
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
    }

    let Some(block_id) = item.as_block_id() else { return; };
    let Some(runtime_id) = registry.get_id_by_identifier(block_id) else { return; };
    let layer = registry.get_layer(runtime_id, 4);
    let atlas_index = layer_to_atlas_index(layer);

    parent.spawn((
        SlotIcon,
        ImageNode {
            image: registry.base_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: registry.atlas_layout.clone(),
                index: atlas_index,
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

/// 原地更新槽位图标，不销毁槽位实体
pub fn sync_slot_icon(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    registry: &BlockRegistry,
    children_query: &Query<&Children>,
) {
    let Ok(children) = children_query.get(slot_entity) else { return; };
    let Some(&icon_entity) = children.first() else { return; };

    if item.is_air() {
        commands.entity(icon_entity).insert(Visibility::Hidden);
        return;
    }

    let Some(block_id) = item.as_block_id() else { return; };
    let Some(runtime_id) = registry.get_id_by_identifier(block_id) else { return; };
    let layer = registry.get_layer(runtime_id, 4);
    let atlas_index = layer_to_atlas_index(layer);

    commands.entity(icon_entity).insert((
        Visibility::Visible,
        ImageNode {
            image: registry.base_texture.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: registry.atlas_layout.clone(),
                index: atlas_index,
            }),
            ..default()
        },
    ));
}