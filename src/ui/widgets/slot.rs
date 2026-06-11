use bevy::prelude::*;
use crate::core::constant::world::CHUNK_SIZE;
use crate::inventory::item::id::ItemId;
use crate::ui::theme::ui_theme::UiTheme;
use crate::voxel::registry::BlockRegistry;

/// =========================
/// Components
/// =========================

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

#[derive(Component)]
pub struct SlotIcon;

#[derive(Component)]
pub struct CategoryTab {
    pub category_index: usize,
}

#[derive(Component)]
pub struct CreativeSearchInput;

#[derive(Resource, Default)]
pub struct SearchInputState {
    pub active: bool,
}

/// =========================
/// Events
/// =========================

#[derive(Message, Debug)]
pub struct SlotClickedEvent {
    pub kind: SlotKind,
    pub index: usize,
}

#[derive(Message, Debug)]
pub struct CategoryClickedEvent {
    pub category_index: usize,
}

/// =========================
/// Slot Spawn
/// =========================

pub fn spawn_empty_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
) {
    spawn_slot_internal(
        parent,
        kind,
        index,
        None,
        None,
        theme,
    );
}

pub fn spawn_slot_with_item(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    item: &ItemId,
    registry: &BlockRegistry,
    theme: &UiTheme,
) {
    spawn_slot_internal(
        parent,
        kind,
        index,
        Some(item),
        Some(registry),
        theme,
    );
}

fn spawn_slot_internal(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    item: Option<&ItemId>,
    registry: Option<&BlockRegistry>,
    theme: &UiTheme,
) {
    parent.spawn((
        InventorySlot { kind, index },
        Button,
        Pickable::default(),
        Node {
            width: Val::Px(theme.slot_size),
            height: Val::Px(theme.slot_size),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(
                Val::Px(theme.slot_border),
            ),
            ..default()
        },
        BackgroundColor(theme.bg_slot),
        BorderColor::all(theme.border_default),
    ))
        .with_children(|slot| {
            match (item, registry) {
                (Some(item), Some(registry)) => {
                    spawn_slot_icon(
                        slot,
                        item,
                        registry,
                    );
                }
                _ => {
                    slot.spawn((
                        SlotIcon,
                        Node {
                            width: Val::Percent(80.0),
                            height: Val::Percent(80.0),
                            ..default()
                        },
                        Visibility::Hidden,
                    ));
                }
            }
        });
}

/// =========================
/// Icon
/// =========================

pub fn spawn_slot_icon(
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

    let Some(block_id) = item.as_block_id() else {
        return;
    };

    let Some(runtime_id) =
        registry.get_id_by_identifier(block_id)
    else {
        return;
    };

    let layer =
        registry.get_layer(runtime_id, 4);

    let atlas_index =
        layer_to_atlas_index(layer);

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

/// =========================
/// Runtime Update
/// =========================

pub fn update_slot_item(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    registry: &BlockRegistry,
    children_query: &Query<&Children>,
) {
    let Ok(children) = children_query.get(slot_entity)
    else {
        return;
    };

    let Some(icon_entity) = children.first().copied()
    else {
        return;
    };

    if item.is_air() {
        commands.entity(icon_entity).insert((
            Visibility::Hidden,
        ));
        return;
    }

    let Some(block_id) = item.as_block_id()
    else {
        return;
    };

    let Some(runtime_id) =
        registry.get_id_by_identifier(block_id)
    else {
        return;
    };

    let layer =
        registry.get_layer(runtime_id, 4);

    let atlas_index =
        layer_to_atlas_index(layer);

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

/// 兼容旧代码
pub fn apply_item_texture(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    registry: &BlockRegistry,
    children_query: &Query<&Children>,
    _image_query: &mut Query<
        (&mut ImageNode, &mut BackgroundColor)
    >,
) {
    update_slot_item(
        commands,
        slot_entity,
        item,
        registry,
        children_query,
    );
}

/// =========================
/// Atlas Helper
/// =========================

fn layer_to_atlas_index(
    layer_idx: u32,
) -> usize {
    (layer_idx as usize)
        * CHUNK_SIZE
        * CHUNK_SIZE
}