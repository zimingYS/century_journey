use bevy::prelude::*;

use crate::client::renderer::item_model::ItemModelRenderAssets;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::navigation::{UiNavigation, UiScreen};
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::crafting::grid::{
    ActiveCrafting, CraftingGrid, PlayerCrafting, WorkbenchCrafting,
};
use crate::game::crafting::plugin::CraftingStationOpened;
use crate::game::inventory::container::InventoryContainer;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::ContainerKind;

const CRAFTING_SLOT_SIZE: f32 = 42.0;

#[derive(Component)]
pub struct CraftingPanel {
    kind: ContainerKind,
}

#[derive(Component)]
pub struct CraftingHost;

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
        spawn_crafting_panel(
            root,
            ContainerKind::Workbench,
            "工作台",
            WorkbenchCrafting::WIDTH,
            WorkbenchCrafting::HEIGHT,
            false,
            &theme,
            &ui_font,
        );
    });
}

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

pub fn open_crafting_station_ui_system(
    mut reader: MessageReader<CraftingStationOpened>,
    mut navigation: MessageWriter<UiNavigation>,
) {
    if reader.read().next().is_some() {
        navigation.write(UiNavigation::Open(UiScreen::Container));
    }
}

pub fn sync_crafting_panel_system(
    active: Res<ActiveCrafting>,
    mut panels: Query<(&CraftingPanel, &mut Node)>,
) {
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

pub fn crafting_visual_sync_system(
    player_crafting: Res<PlayerCrafting>,
    workbench_crafting: Res<WorkbenchCrafting>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
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
    for (entity, slot, mut visual) in &mut slot_query {
        let SlotKind::Container(kind) = slot.kind else {
            continue;
        };
        let current = match kind {
            ContainerKind::PlayerCrafting => {
                crafting_slot_value(player_crafting.grid(), slot.index)
            }
            ContainerKind::Workbench => crafting_slot_value(workbench_crafting.grid(), slot.index),
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
                &item_model_assets,
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
