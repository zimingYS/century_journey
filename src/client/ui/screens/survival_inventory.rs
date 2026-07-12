use bevy::camera::{RenderTarget, ScalingMode, visibility::RenderLayers};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

use crate::client::renderer::item_model::ItemModelRenderAssets;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::components::{
    CompactBackpackButton, SortBackpackButton, SurvivalAccessoryPanel, SurvivalDefenseText,
    SurvivalEquipmentPanel, SurvivalHealthText, SurvivalHotbarPanel, SurvivalHungerText,
    SurvivalInventoryOverlay, SurvivalInventoryRoot, SurvivalItemGrid, SurvivalPlayerPreviewCamera,
};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::screens::crafting::CraftingHost;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, spawn_empty_slot_with_placeholder,
    sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::gameplay::gamemode::PlayerGameMode;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::equipment::{AccessorySlotDefinitions, EquipmentSlot};
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::state::InventoryState;
use crate::game::player::components::LocalPlayer;
use crate::game::player::components::stats::{Defense, Health, Hunger};
use crate::game::player::model::config::PlayerModelConfig;
use crate::shared::item_id::ItemId;

const SURVIVAL_PANEL_WIDTH: f32 = 708.0;
const SURVIVAL_PANEL_HEIGHT: f32 = 680.0;
const MAIN_SLOT_SIZE: f32 = 54.0;
const SIDE_SLOT_SIZE: f32 = 40.0;
const SIDE_PANEL_WIDTH: f32 = 58.0;
const PREVIEW_LAYER: usize = 7;
/// 生存物品栏位于 HUD 之上，避免被快捷栏和状态条遮挡。
const SURVIVAL_OVERLAY_Z: i32 = 1000;

pub fn spawn_survival_inventory_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    accessory_definitions: Res<AccessorySlotDefinitions>,
    mut inventory: ResMut<InventoryState>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_model_config: Res<PlayerModelConfig>,
) {
    inventory
        .survival
        .ensure_accessory_slots(accessory_definitions.slots.len());
    let preview_image = spawn_player_preview(
        &mut commands,
        &mut images,
        &mut meshes,
        &mut materials,
        &player_model_config,
    );

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
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            ZIndex(SURVIVAL_OVERLAY_Z),
            BackgroundColor(Color::srgba(0.015, 0.02, 0.025, 0.78)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    SurvivalInventoryRoot,
                    Name::new("SurvivalRoot"),
                    Node {
                        width: Val::Px(SURVIVAL_PANEL_WIDTH),
                        height: Val::Px(SURVIVAL_PANEL_HEIGHT),
                        max_width: Val::Percent(100.0),
                        max_height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(10.0)),
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.075, 0.075, 0.085, 0.98)),
                    BorderColor::all(Color::srgba(0.38, 0.38, 0.42, 1.0)),
                ))
                .with_children(|root| {
                    root.spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(354.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|top| {
                        build_equipment_panel(top, &theme, &ui_font);
                        build_preview_panel(top, preview_image.clone(), &theme, &ui_font);
                        build_accessory_panel(top, &accessory_definitions, &theme, &ui_font);
                    });

                    build_backpack_panel(root, &theme, &ui_font);
                    build_survival_hotbar_panel(root, &theme);
                });
        });
}

fn build_equipment_panel(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    let side_theme = slot_theme(theme, SIDE_SLOT_SIZE);
    parent
        .spawn((
            SurvivalEquipmentPanel,
            Node {
                width: Val::Px(SIDE_PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_sidebar),
            BorderColor::all(Color::srgba(0.1, 0.76, 0.7, 0.8)),
        ))
        .with_children(|panel| {
            spawn_heading(panel, "装备", theme, ui_font);
            for (index, equipment_slot) in EquipmentSlot::ALL.into_iter().enumerate() {
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(SIDE_SLOT_SIZE),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(9.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_empty_slot_with_placeholder(
                            row,
                            SlotKind::SurvivalEquipment,
                            index,
                            equipment_slot.placeholder(),
                            &side_theme,
                            ui_font,
                        );
                    });
            }
        });
}

fn build_accessory_panel(
    parent: &mut ChildSpawnerCommands,
    definitions: &AccessorySlotDefinitions,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    let side_theme = slot_theme(theme, SIDE_SLOT_SIZE);
    parent
        .spawn((
            SurvivalAccessoryPanel,
            Node {
                width: Val::Px(SIDE_PANEL_WIDTH),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(5.0),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_sidebar),
            BorderColor::all(Color::srgba(0.92, 0.7, 0.08, 0.9)),
        ))
        .with_children(|panel| {
            spawn_heading(panel, "饰品", theme, ui_font);
            for (index, definition) in definitions.slots.iter().enumerate() {
                panel
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(SIDE_SLOT_SIZE),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(9.0),
                        ..default()
                    })
                    .with_children(|row| {
                        spawn_empty_slot_with_placeholder(
                            row,
                            SlotKind::SurvivalAccessory,
                            index,
                            &definition.placeholder,
                            &side_theme,
                            ui_font,
                        );
                    });
            }
        });
}

fn build_preview_panel(
    parent: &mut ChildSpawnerCommands,
    preview_image: Handle<Image>,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn((
            CraftingHost,
            Node {
                flex_grow: 1.0,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ))
        .with_children(|center| {
            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_grow: 1.0,
                        height: Val::Percent(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.13, 0.15, 1.0)),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|preview| {
                    preview.spawn((
                        ImageNode {
                            image: preview_image,
                            ..default()
                        },
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                    ));
                });

            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        justify_content: JustifyContent::SpaceAround,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_content),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|stats| {
                    spawn_stat_text::<SurvivalHealthText>(
                        stats,
                        "生命 --",
                        Color::srgb(0.96, 0.25, 0.24),
                        ui_font,
                    );
                    spawn_stat_text::<SurvivalDefenseText>(
                        stats,
                        "防御 --",
                        Color::srgb(0.55, 0.7, 0.82),
                        ui_font,
                    );
                    spawn_stat_text::<SurvivalHungerText>(
                        stats,
                        "饥饿 --",
                        Color::srgb(0.88, 0.55, 0.25),
                        ui_font,
                    );
                });

            center
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(42.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexEnd,
                        column_gap: Val::Px(7.0),
                        padding: UiRect::horizontal(Val::Px(7.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(theme.bg_content),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|actions| {
                    spawn_action_button::<CompactBackpackButton>(actions, "收拢", theme, ui_font);
                    spawn_action_button::<SortBackpackButton>(actions, "整理", theme, ui_font);
                });
        });
}

fn build_backpack_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    let grid_width = MAIN_SLOT_SIZE * 9.0 + theme.slot_gap * 8.0 + 12.0;
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(190.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(5.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.055, 0.055, 0.065, 1.0)),
    ))
    .with_children(|section| {
        section
            .spawn(Node {
                width: Val::Px(grid_width),
                height: Val::Px(19.0),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|title| spawn_label(title, "背包", theme, ui_font));
        section.spawn((
            SurvivalItemGrid,
            Name::new("SurvivalGrid"),
            Node {
                width: Val::Px(grid_width),
                height: Val::Px(MAIN_SLOT_SIZE * 3.0 + theme.slot_gap * 2.0 + 12.0),
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::px(9, MAIN_SLOT_SIZE),
                grid_template_rows: RepeatedGridTrack::px(3, MAIN_SLOT_SIZE),
                column_gap: Val::Px(theme.slot_gap),
                row_gap: Val::Px(theme.slot_gap),
                padding: UiRect::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.bg_content),
            BorderColor::all(theme.border_default),
        ));
    });
}

fn build_survival_hotbar_panel(root: &mut ChildSpawnerCommands, theme: &UiTheme) {
    let width = MAIN_SLOT_SIZE * 9.0 + theme.slot_gap * 8.0 + 12.0;
    root.spawn((
        SurvivalHotbarPanel,
        Node {
            width: Val::Px(width),
            height: Val::Px(66.0),
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(1.0)),
            padding: UiRect::all(Val::Px(5.0)),
            column_gap: Val::Px(theme.slot_gap),
            ..default()
        },
        BackgroundColor(theme.bg_content),
        BorderColor::all(Color::srgba(0.4, 0.78, 0.25, 0.85)),
    ));
}

fn spawn_heading(
    parent: &mut ChildSpawnerCommands,
    value: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent.spawn((
        Text::new(value.to_string()),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(17.0),
            ..default()
        },
        TextColor(theme.text_primary),
        Node {
            height: Val::Px(22.0),
            ..default()
        },
    ));
}

fn spawn_label(parent: &mut ChildSpawnerCommands, value: &str, theme: &UiTheme, ui_font: &UiFont) {
    parent.spawn((
        Text::new(value.to_string()),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(theme.body_font_size),
            ..default()
        },
        TextColor(theme.text_secondary),
    ));
}

fn spawn_stat_text<M: Component + Default>(
    parent: &mut ChildSpawnerCommands,
    value: &str,
    color: Color,
    ui_font: &UiFont,
) {
    parent.spawn((
        M::default(),
        Text::new(value.to_string()),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(color),
    ));
}

fn spawn_action_button<M: Component + Default>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    parent
        .spawn((
            M::default(),
            Button,
            Pickable::default(),
            Node {
                width: Val::Px(72.0),
                height: Val::Px(29.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(theme.tab_active_bg),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|button| {
            button.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(theme.text_primary),
            ));
        });
}

fn slot_theme(theme: &UiTheme, size: f32) -> UiTheme {
    let mut result = theme.clone();
    result.slot_size = size;
    result.slot_border = 1.0;
    result
}

fn spawn_player_preview(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    config: &PlayerModelConfig,
) -> Handle<Image> {
    let image = Image::new_target_texture(384, 320, TextureFormat::Rgba8UnormSrgb, None);
    let image_handle = images.add(image);
    let target = Vec3::new(0.0, -750.0, 0.0);
    let preview_layer = RenderLayers::layer(PREVIEW_LAYER);
    let (root, rig) =
        crate::game::player::model::rig::spawn_player_rig_v2(commands, meshes, materials, config);

    commands.entity(root).insert((
        Transform {
            translation: target,
            rotation: Quat::from_rotation_y(std::f32::consts::PI),
            ..default()
        },
        preview_layer.clone(),
        Name::new("InventoryPlayerPreview"),
    ));
    for entity in rig.mesh_entities {
        commands
            .entity(entity)
            .insert((preview_layer.clone(), NotShadowCaster));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(target + Vec3::new(3.0, 4.0, 4.0)).looking_at(target, Vec3::Y),
        preview_layer.clone(),
        Name::new("InventoryPreviewLight"),
    ));

    commands.spawn((
        SurvivalPlayerPreviewCamera,
        Camera3d::default(),
        Camera {
            order: -8,
            is_active: false,
            clear_color: Color::NONE.into(),
            ..default()
        },
        RenderTarget::Image(image_handle.clone().into()),
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::Fixed {
                width: 2.8,
                height: 2.6,
            },
            near: 0.0,
            far: 32.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_translation(target + Vec3::new(2.8, 1.35, 4.6))
            .looking_at(target + Vec3::Y * 0.12, Vec3::Y),
        preview_layer,
        Name::new("InventoryPreviewCamera"),
    ));

    image_handle
}

pub fn update_survival_visibility_system(
    state: Res<InventoryState>,
    gamemode: Res<PlayerGameMode>,
    mut overlay_query: Query<&mut Visibility, With<SurvivalInventoryOverlay>>,
    mut camera_query: Query<&mut Camera, With<SurvivalPlayerPreviewCamera>>,
) {
    let visible = state.opened && gamemode.is_survival();
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = if visible {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut camera) = camera_query.single_mut() {
        camera.is_active = visible;
    }
}

/// 存档恢复发生在 Startup 之后，因此每帧只做廉价的扩容检查。
pub fn sync_accessory_slot_count_system(
    definitions: Res<AccessorySlotDefinitions>,
    mut state: ResMut<InventoryState>,
) {
    state
        .survival
        .ensure_accessory_slots(definitions.slots.len());
}

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

    let slot_theme = slot_theme(&theme, MAIN_SLOT_SIZE);
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

pub fn survival_grid_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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
    let revision = item_model_assets.revision();
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
        let Some(index) = crate::game::inventory::routing::survival_index(slot.kind, slot.index)
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
                &item_model_assets,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
            visual.item = item;
            visual.count = count;
        }
    }
}

pub fn survival_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    children_query: Query<&Children>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
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
    let revision = item_model_assets.revision();
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
                    &item_model_assets,
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
    }
}

pub fn init_survival_hotbar_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
    hotbar_query: Query<Entity, With<SurvivalHotbarPanel>>,
    children_query: Query<&Children>,
    slot_query: Query<&InventorySlot>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
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

    let slot_theme = slot_theme(&theme, MAIN_SLOT_SIZE);
    commands.entity(panel_entity).with_children(|bar| {
        for (index, item) in state.hotbar.items().iter().enumerate() {
            crate::client::ui::widgets::slot::spawn_slot_with_item(
                bar,
                SlotKind::Hotbar,
                index,
                item,
                registry,
                render_assets,
                &item_model_assets,
                &slot_theme,
                &ui_font,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
        }
    });
}

pub fn cleanup_survival_hotbar_system() {}

pub fn survival_stats_visual_sync_system(
    player_query: Query<(&Health, &Hunger, Option<&Defense>), With<LocalPlayer>>,
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
            *text = Text::new(format!("生命 {:.0}/{:.0}", health.current, health.max));
        } else if defense_marker.is_some() {
            *text = Text::new(format!("防御 {:.0}", defense.map_or(0.0, |value| value.0)));
        } else if hunger_marker.is_some() {
            *text = Text::new(format!("饥饿 {:.0}/{:.0}", hunger.current, hunger.max));
        }
    }
}

pub fn backpack_management_button_system(
    mut writer: MessageWriter<crate::game::inventory::events::InventoryCommand>,
    mut query: Query<
        (
            &Interaction,
            Option<&CompactBackpackButton>,
            Option<&SortBackpackButton>,
            &mut BackgroundColor,
        ),
        (
            Changed<Interaction>,
            With<Button>,
            Or<(With<CompactBackpackButton>, With<SortBackpackButton>)>,
        ),
    >,
    theme: Res<UiTheme>,
) {
    for (interaction, compact, sort, mut background) in &mut query {
        *background = BackgroundColor(match interaction {
            Interaction::Hovered => Color::srgba(0.28, 0.3, 0.36, 1.0),
            Interaction::Pressed => theme.accent,
            Interaction::None => theme.tab_active_bg,
        });
        if *interaction != Interaction::Pressed {
            continue;
        }
        if compact.is_some() {
            writer.write(crate::game::inventory::events::InventoryCommand::CompactBackpack);
        } else if sort.is_some() {
            writer.write(crate::game::inventory::events::InventoryCommand::SortBackpack);
        }
    }
}

pub fn handle_inventory_close(state: &mut InventoryState) {
    use crate::game::inventory::cursor::CursorSource;
    if !state.cursor.has_item() {
        return;
    }

    let mut remaining = state.cursor.stack().cloned().unwrap();
    if let Some(source) = state.cursor.source {
        match source {
            CursorSource::Hotbar(index) => {
                remaining = return_to_container(&mut state.hotbar, index, remaining);
            }
            CursorSource::SurvivalBackpack(index) => {
                remaining = return_to_container(&mut state.survival, index, remaining);
            }
            _ => {}
        }
    }
    if remaining.is_empty() {
        state.cursor.clear();
        return;
    }

    let active = state.hotbar.active_index;
    remaining = return_to_container(&mut state.hotbar, active, remaining);
    for index in 0..HOTBAR_SIZE {
        if remaining.is_empty() {
            break;
        }
        remaining = return_to_container(&mut state.hotbar, index, remaining);
    }
    for index in 0..SurvivalInventory::BACKPACK_SIZE {
        if remaining.is_empty() {
            break;
        }
        remaining = return_to_container(&mut state.survival, index, remaining);
    }

    state.cursor.clear();
    if !remaining.is_empty() {
        log::warn!("[Survival] backpack full, lost: {remaining:?}");
    }
}

fn return_to_container<C: InventoryContainer>(
    container: &mut C,
    index: usize,
    mut remaining: ItemStack,
) -> ItemStack {
    if remaining.is_empty() {
        return remaining;
    }
    if let Some(stack) = container.get_stack_mut(index)
        && stack.item == remaining.item
    {
        stack.merge_from(&mut remaining);
    }
    if !remaining.is_empty() && container.get_stack(index).is_none() {
        container.set_stack(index, remaining);
        return ItemStack::empty();
    }
    remaining
}
