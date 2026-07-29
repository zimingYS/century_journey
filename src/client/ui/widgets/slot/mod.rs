pub mod components;

pub use crate::shared::ui_types::{SearchInputState, SlotKind};
pub use components::{
    CategoryClickedEvent, CategoryTab, CreativeSearchInput, InventorySlot, SlotCountText,
    SlotDurabilityBar, SlotDurabilityFill, SlotIcon, SlotInteractionEvent, SlotPlaceholder,
    SlotVisual,
};

use crate::client::renderer::item_model::{ItemModelRenderAssets, ItemModelRenderer};
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::crafting::grid::ActiveCrafting;
use crate::game::crafting::grid::{CraftingGrid, PlayerCrafting, WorkbenchCrafting};
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::LocalPlayer;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::ContainerKind;
use bevy::prelude::*;

/// 鐢熸垚绌烘Ы浣嶃€?
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
            spawn_durability_bar(slot, kind, index);
        });
}

/// 鐢熸垚甯︾煭鍗犱綅鏍囪鐨勭┖妲戒綅锛岀敤浜庤澶囨爮鍜岄グ鍝佹爮銆?
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
            spawn_durability_bar(slot, kind, index);
        });
}

/// 鐢熸垚甯︾墿鍝佸浘鏍囩殑妲戒綅銆?
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
            spawn_durability_bar(slot, kind, index);
        });
}

/// 鐢熸垚妲戒綅鍥炬爣瀛愯妭鐐广€?
///
/// UI 灞備笉鍒ゆ柇鏂瑰潡鎴栬创鍥剧被鍨嬶紝鍙悜 ItemRenderer 鏌ヨ褰撳墠鐗╁搧鍦?GUI 涓簲璇ユ樉绀虹殑鍥剧墖锛?
/// 褰?3D 鏂瑰潡鍥炬爣浠嶅湪绂诲睆鐑樼剻鏃讹紝涓存椂鍥為€€鍒版柟鍧?atlas 鍥炬爣锛岄伩鍏嶅嚭鐜扮┖妲姐€?
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

/// 鍘熷湴鍚屾妲戒綅鍥炬爣鍜屾暟閲忔枃鏈€?
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

/// 鍒涘缓缁熶竴灏哄鐨勬Ы浣嶅浘鏍囪妭鐐广€?
fn icon_node() -> Node {
    Node {
        width: Val::Percent(80.0),
        height: Val::Percent(80.0),
        ..default()
    }
}

/// 鍒涘缓鏅€氬浘鐗囪妭鐐广€?
fn plain_image_node(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        texture_atlas: None,
        ..default()
    }
}

fn spawn_durability_bar(parent: &mut ChildSpawnerCommands, kind: SlotKind, index: usize) {
    parent
        .spawn((
            SlotDurabilityBar { kind, index },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(3.0),
                right: Val::Px(3.0),
                bottom: Val::Px(2.0),
                height: Val::Px(4.0),
                padding: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.03, 0.03, 0.035)),
            Visibility::Hidden,
        ))
        .with_children(|bar| {
            bar.spawn((
                SlotDurabilityFill,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.9, 0.2)),
            ));
        });
}

pub fn sync_slot_durability_system(
    player_query: Query<(&InventoryState, &PlayerCrafting, &ActiveCrafting), With<LocalPlayer>>,
    containers: Res<WorldContainers>,
    item_registry: Option<Res<ItemRegistry>>,
    mut bar_query: Query<(&SlotDurabilityBar, &Children, &mut Visibility)>,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor), With<SlotDurabilityFill>>,
) {
    let Some(item_registry) = item_registry else {
        return;
    };
    let Ok((inventory, player_crafting, active)) = player_query.single() else {
        return;
    };
    let workbench = active.container_id.and_then(|id| containers.workbench(id));
    for (bar, children, mut visibility) in &mut bar_query {
        let Some(stack) = stack_for_slot(bar, inventory, player_crafting, workbench) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let Some(max_durability) = item_registry
            .get(&stack.item)
            .and_then(|definition| definition.tool_data())
            .map(|tool| tool.max_durability)
            .filter(|max| *max > 0)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };
        let remaining = stack
            .durability()
            .unwrap_or(max_durability)
            .min(max_durability);
        if remaining >= max_durability {
            *visibility = Visibility::Hidden;
            continue;
        }

        let Some(&fill_entity) = children.first() else {
            continue;
        };
        let Ok((mut fill, mut color)) = fill_query.get_mut(fill_entity) else {
            continue;
        };
        let ratio = remaining as f32 / max_durability as f32;
        fill.width = Val::Percent((ratio * 100.0).clamp(0.0, 100.0));
        *color = BackgroundColor(Color::srgb(1.0 - ratio, 0.15 + ratio * 0.75, 0.06));
        *visibility = Visibility::Inherited;
    }
}

fn stack_for_slot<'a>(
    bar: &SlotDurabilityBar,
    inventory: &'a InventoryState,
    player_crafting: &'a PlayerCrafting,
    workbench_crafting: Option<&'a WorkbenchCrafting>,
) -> Option<&'a ItemStack> {
    match bar.kind {
        SlotKind::Hotbar => inventory.hotbar.get_stack(bar.index),
        SlotKind::SurvivalBackpack | SlotKind::SurvivalEquipment | SlotKind::SurvivalAccessory => {
            let index =
                crate::game::inventory::interaction::routing::survival_index(bar.kind, bar.index)?;
            inventory.survival.get_stack(index)
        }
        SlotKind::Container(ContainerKind::PlayerCrafting) => {
            crafting_stack(player_crafting.grid(), bar.index)
        }
        SlotKind::Container(ContainerKind::Workbench) => {
            workbench_crafting.and_then(|workbench| crafting_stack(workbench.grid(), bar.index))
        }
        SlotKind::CreativeGrid
        | SlotKind::Recent
        | SlotKind::Container(ContainerKind::Chest | ContainerKind::Furnace) => None,
    }
}

fn crafting_stack(grid: &CraftingGrid, index: usize) -> Option<&ItemStack> {
    if index < grid.slot_count() {
        grid.get_stack(index)
    } else if index == grid.slot_count() {
        grid.output()
    } else {
        None
    }
}

/// 鍦?3D 鏂瑰潡鍥炬爣灏氭湭 ready 鏃讹紝鍥為€€鍒版柟鍧?atlas 閲岀殑 2D 鍥炬爣銆?
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

/// 鍚屾蹇嵎鏍忛潰鏉跨殑妲戒綅鍥炬爣銆佹暟閲忓拰閫変腑杈规銆?
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

/// 鐢熸垚浠呭睍绀虹敤妲戒綅銆?
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
            spawn_durability_bar(slot, kind, index);
        });
}
