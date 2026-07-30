//! 构建工作台界面并同步当前合成容器的可见槽位。

use bevy::prelude::*;

use crate::client::renderer::item::GuiItemIconCache;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::navigation::{UiNavigation, UiScreen, UiScreenRoot};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::crafting::events::CraftingStationOpened;
use crate::game::crafting::grid::{
    ActiveCrafting, CraftingGrid, PlayerCrafting, WorkbenchCrafting,
};
use crate::game::inventory::container::ContainerKind;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::container::survival::SurvivalInventory;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::player::identity::{LocalPlayer, PlayerId};
use crate::shared::item_id::ItemId;

const CRAFTING_SLOT_SIZE: f32 = 42.0;
const CONTAINER_SLOT_SIZE: f32 = 46.0;
const WORKBENCH_PANEL_WIDTH: f32 = 580.0;
const WORKBENCH_PANEL_HEIGHT: f32 = 510.0;

/// 工作台屏幕中的合成槽位面板。
#[derive(Component)]
pub struct CraftingPanel {
    kind: ContainerKind,
}

/// 承载合成界面的世界容器宿主标记。
#[derive(Component)]
pub struct CraftingHost;

/// 工作台合成界面的遮罩根节点。
#[derive(Component)]
pub struct WorkbenchOverlay;

/// 创建玩家随身合成和工作台合成界面节点。
pub fn spawn_crafting_system(
    roots: Query<Entity, With<CraftingHost>>,
    panels: Query<(), With<CraftingPanel>>,
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
) {
    let Ok(root) = roots.single() else { return };
    if !panels.is_empty() {
        return;
    }
    commands.entity(root).with_children(|root| {
        spawn_crafting_panel(
            root,
            ContainerKind::PlayerCrafting,
            "随身合成",
            PlayerCrafting::WIDTH,
            PlayerCrafting::HEIGHT,
            true,
            &theme,
            &ui_font,
        );
    });
    spawn_workbench_overlay(&mut commands, &theme, &ui_font);
}

fn spawn_workbench_overlay(commands: &mut Commands, theme: &UiTheme, ui_font: &UiFont) {
    commands
        .spawn((
            WorkbenchOverlay,
            UiScreenRoot::new(UiScreen::Container),
            Name::new("WorkbenchOverlay"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            ZIndex(1_100),
            BackgroundColor(Color::srgba(0.015, 0.02, 0.025, 0.76)),
            Visibility::Hidden,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Name::new("WorkbenchRoot"),
                    Node {
                        width: Val::Px(WORKBENCH_PANEL_WIDTH),
                        height: Val::Px(WORKBENCH_PANEL_HEIGHT),
                        max_width: Val::Percent(100.0),
                        max_height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(14.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.075, 0.075, 0.085, 0.99)),
                    BorderColor::all(theme.border_default),
                ))
                .with_children(|root| {
                    spawn_crafting_panel(
                        root,
                        ContainerKind::Workbench,
                        "工作台",
                        WorkbenchCrafting::WIDTH,
                        WorkbenchCrafting::HEIGHT,
                        false,
                        theme,
                        ui_font,
                    );
                    spawn_player_storage(root, theme, ui_font);
                });
        });
}

fn spawn_player_storage(parent: &mut ChildSpawnerCommands, theme: &UiTheme, ui_font: &UiFont) {
    parent.spawn((
        Text::new("物品栏"),
        TextFont {
            font: FontSource::from(ui_font.default.clone()),
            font_size: FontSize::Px(theme.body_font_size),
            ..default()
        },
        TextColor(theme.text_primary),
    ));

    let mut slot_theme = theme.clone();
    slot_theme.slot_size = CONTAINER_SLOT_SIZE;
    parent
        .spawn(Node {
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::px(9, CONTAINER_SLOT_SIZE),
            grid_template_rows: RepeatedGridTrack::px(3, CONTAINER_SLOT_SIZE),
            column_gap: Val::Px(4.0),
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        })
        .with_children(|grid| {
            for index in 0..SurvivalInventory::BACKPACK_SIZE {
                spawn_empty_slot(
                    grid,
                    SlotKind::SurvivalBackpack,
                    index,
                    &slot_theme,
                    ui_font,
                );
            }
        });

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        })
        .with_children(|hotbar| {
            for index in 0..HOTBAR_SIZE {
                spawn_empty_slot(hotbar, SlotKind::Hotbar, index, &slot_theme, ui_font);
            }
        });
}

// 面板的布局规格只在生成时使用，显式参数比创建一次性配置类型更容易核对。
#[allow(clippy::too_many_arguments)]
fn spawn_crafting_panel(
    parent: &mut ChildSpawnerCommands,
    kind: ContainerKind,
    title: &str,
    columns: usize,
    rows: usize,
    visible: bool,
    theme: &UiTheme,
    ui_font: &UiFont,
) {
    let grid_height = rows as f32 * CRAFTING_SLOT_SIZE + rows.saturating_sub(1) as f32 * 4.0;
    parent
        .spawn((
            CraftingPanel { kind },
            Node {
                display: if visible {
                    Display::Flex
                } else {
                    Display::None
                },
                width: Val::Percent(100.0),
                height: Val::Px(grid_height + 18.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(14.0),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(theme.bg_content),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(theme.body_font_size),
                    ..default()
                },
                TextColor(theme.text_primary),
            ));

            let mut slot_theme = theme.clone();
            slot_theme.slot_size = CRAFTING_SLOT_SIZE;
            panel
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::px(
                        columns as u16,
                        CRAFTING_SLOT_SIZE,
                    ),
                    grid_template_rows: RepeatedGridTrack::px(rows as u16, CRAFTING_SLOT_SIZE),
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|grid| {
                    for index in 0..columns * rows {
                        spawn_empty_slot(
                            grid,
                            SlotKind::Container(kind),
                            index,
                            &slot_theme,
                            ui_font,
                        );
                    }
                });

            panel.spawn((
                Text::new("→"),
                TextFont {
                    font: FontSource::from(ui_font.default.clone()),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(theme.text_secondary),
            ));

            spawn_empty_slot(
                panel,
                SlotKind::Container(kind),
                columns * rows,
                &slot_theme,
                ui_font,
            );
        });
}

/// 在工作台容器打开后把对应界面压入导航栈。
pub fn open_crafting_station_ui_system(
    mut reader: MessageReader<CraftingStationOpened>,
    mut navigation: MessageWriter<UiNavigation>,
    local_player: Single<&PlayerId, With<LocalPlayer>>,
) {
    if reader.read().any(|event| event.player_id == **local_player) {
        navigation.write(UiNavigation::Open(UiScreen::Container));
    }
}

/// 根据当前活动合成容器同步面板布局和槽位绑定。
pub fn sync_crafting_panel_system(
    active_query: Query<Ref<ActiveCrafting>, With<LocalPlayer>>,
    mut panels: Query<(&CraftingPanel, &mut Node)>,
) {
    let Ok(active) = active_query.single() else {
        return;
    };
    if !active.is_changed() {
        return;
    }
    for (panel, mut node) in &mut panels {
        node.display = if panel.kind == active.kind {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// 把权威合成槽位内容同步到现有界面槽位表现。
/// 多类内容缓存只用于生成槽位表现，保持显式参数可审查 Client 与 Game 的边界。
#[allow(clippy::too_many_arguments)]
pub fn crafting_visual_sync_system(
    player_query: Query<(&PlayerCrafting, &ActiveCrafting), With<LocalPlayer>>,
    containers: Res<WorldContainers>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    gui_item_icons: Res<GuiItemIconCache>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
    children_query: Query<&Children>,
    mut slot_query: Query<(Entity, &InventorySlot, &mut SlotVisual)>,
    mut commands: Commands,
) {
    let (Some(block_registry), Some(block_render_assets)) =
        (block_registry.as_deref(), block_render_assets.as_deref())
    else {
        return;
    };
    let Ok((player_crafting, active)) = player_query.single() else {
        return;
    };
    for (entity, slot, mut visual) in &mut slot_query {
        let SlotKind::Container(kind) = slot.kind else {
            continue;
        };
        let current = match kind {
            ContainerKind::PlayerCrafting => {
                crafting_slot_value(player_crafting.grid(), slot.index)
            }
            ContainerKind::Workbench => active
                .container_id
                .and_then(|id| containers.workbench(id))
                .map(|workbench| crafting_slot_value(workbench.grid(), slot.index))
                .unwrap_or((ItemId::air(), 0)),
            ContainerKind::Chest | ContainerKind::Furnace => continue,
        };
        if visual.item != current.0 || visual.count != current.1 {
            sync_slot_icon(
                &mut commands,
                entity,
                &current.0,
                current.1,
                block_registry,
                block_render_assets,
                &gui_item_icons,
                &children_query,
                item_registry.as_deref(),
                item_texture_registry.as_deref(),
            );
            visual.item = current.0;
            visual.count = current.1;
        }
    }
}

fn crafting_slot_value(grid: &CraftingGrid, index: usize) -> (ItemId, u32) {
    if index < grid.slot_count() {
        grid.get_stack(index)
            .map(|stack| (stack.item.clone(), stack.count))
            .unwrap_or((ItemId::air(), 0))
    } else if index == grid.slot_count() {
        grid.output()
            .map(|stack| (stack.item.clone(), stack.count))
            .unwrap_or((ItemId::air(), 0))
    } else {
        (ItemId::air(), 0)
    }
}
