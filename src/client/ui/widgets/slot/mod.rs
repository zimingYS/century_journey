pub mod components;

pub use crate::shared::ui_types::{SearchInputState, SlotKind};
pub use components::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SlotCountText, SlotIcon,
    SlotInteractionEvent, SlotPlaceholder, SlotVisual,
};

use crate::client::renderer::item_model::{ItemModelRenderAssets, ItemModelRenderer};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// 生成空槽位。
pub fn spawn_empty_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
    ui_font: &UiFont,
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
                    font: FontSource::from(ui_font.default.clone()),
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

/// 生成带短占位标记的空槽位，用于装备栏和饰品栏。
pub fn spawn_empty_slot_with_placeholder(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    placeholder: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
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
                flex_shrink: 0.0,
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
                    font: FontSource::from(ui_font.default.clone()),
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
            slot.spawn((
                SlotPlaceholder,
                Text::new(placeholder.to_string()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.small_font_size),
                    ..default()
                },
                TextColor(theme.text_hint),
            ));
        });
}

/// 生成带物品图标的槽位。
pub fn spawn_slot_with_item(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    item: &ItemId,
    registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
    theme: &UiTheme,
    ui_font: &UiFont,
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
            spawn_icon_child(
                slot,
                item,
                registry,
                render_assets,
                item_model_assets,
                item_registry,
                item_texture_registry,
            );
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
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

/// 生成槽位图标子节点。
///
/// UI 层不判断方块或贴图类型，只向 ItemRenderer 查询当前物品在 GUI 中应该显示的图片；
/// 当 3D 方块图标仍在离屏烘焙时，临时回退到方块 atlas 图标，避免出现空槽。
pub fn spawn_icon_child(
    parent: &mut ChildSpawnerCommands,
    item: &ItemId,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    if let Some(image) = ItemModelRenderer::item_icon_image(
        item,
        item_registry,
        item_texture_registry,
        item_model_assets,
    ) {
        parent.spawn((SlotIcon, plain_image_node(image), icon_node()));
    } else if let Some(image_node) =
        block_atlas_fallback_image(item, block_registry, render_assets, item_registry)
    {
        parent.spawn((SlotIcon, image_node, icon_node()));
    } else {
        parent.spawn((SlotIcon, icon_node(), Visibility::Hidden));
    }
}

/// 原地同步槽位图标和数量文本。
pub fn sync_slot_icon(
    commands: &mut Commands,
    slot_entity: Entity,
    item: &ItemId,
    count: u32,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
    children_query: &Query<&Children>,
    item_registry: Option<&ItemRegistry>,
    item_texture_registry: Option<&ItemTextureRegistry>,
) {
    let Ok(children) = children_query.get(slot_entity) else {
        return;
    };

    if let Some(&icon_entity) = children.first() {
        if item.is_air() {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        } else if let Some(image) = ItemModelRenderer::item_icon_image(
            item,
            item_registry,
            item_texture_registry,
            item_model_assets,
        ) {
            commands
                .entity(icon_entity)
                .insert((Visibility::Inherited, plain_image_node(image)));
        } else if let Some(image_node) =
            block_atlas_fallback_image(item, block_registry, render_assets, item_registry)
        {
            commands
                .entity(icon_entity)
                .insert((Visibility::Inherited, image_node));
        } else {
            commands.entity(icon_entity).insert(Visibility::Hidden);
        }
    }

    if let Some(&count_entity) = children.get(1) {
        if count > 1 {
            commands
                .entity(count_entity)
                .insert((Visibility::Inherited, Text::new(count.to_string())));
        } else {
            commands.entity(count_entity).insert(Visibility::Hidden);
        }
    }

    if let Some(&placeholder_entity) = children.get(2) {
        commands
            .entity(placeholder_entity)
            .insert(if item.is_air() {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
    }
}

/// 创建统一尺寸的槽位图标节点。
fn icon_node() -> Node {
    Node {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        ..default()
    }
}

/// 创建普通图片节点。
fn plain_image_node(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        texture_atlas: None,
        ..default()
    }
}

/// 在 3D 方块图标尚未 ready 时，回退到方块 atlas 里的 2D 图标。
fn block_atlas_fallback_image(
    item: &ItemId,
    block_registry: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_registry: Option<&ItemRegistry>,
) -> Option<ImageNode> {
    let block_id = item_registry
        .and_then(|registry| registry.get(item))
        .and_then(|definition| {
            definition
                .placeable_block
                .as_ref()
                .or_else(|| definition.icon.as_block_id())
        })
        .cloned()
        .unwrap_or_else(|| item.identifier().clone());

    let atlas_index = block_registry.get_icon_atlas_index(&block_id)?;
    Some(ImageNode {
        image: render_assets.base_texture().clone(),
        texture_atlas: Some(TextureAtlas {
            layout: render_assets.atlas_layout().clone(),
            index: atlas_index,
        }),
        ..default()
    })
}

/// 同步快捷栏面板的槽位图标、数量和选中边框。
pub fn sync_hotbar_panel_visuals(
    state: &crate::game::inventory::state::InventoryState,
    reg: &BlockRegistry,
    render_assets: &BlockRenderAssets,
    item_model_assets: &ItemModelRenderAssets,
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

    let revision = item_model_assets.revision();
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
                        item_model_assets,
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

/// 生成仅展示用槽位。
pub fn spawn_display_only_slot(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn((
            InventorySlot { kind, index },
            SlotVisual {
                item: ItemId::air(),
                count: 0,
            },
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
            slot.spawn((SlotIcon, icon_node(), Visibility::Hidden));
            slot.spawn((
                SlotCountText,
                Text::new(""),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
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
