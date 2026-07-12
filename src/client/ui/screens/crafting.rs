use bevy::prelude::*;

use crate::client::renderer::item_model::ItemModelRenderAssets;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_empty_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::crafting::grid::PlayerCrafting;
use crate::game::inventory::container::InventoryContainer;
use crate::shared::item_id::ItemId;

const CRAFTING_SLOT_SIZE: f32 = 42.0;

#[derive(Component)]
pub struct CraftingPanel;

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
        root.spawn((
            CraftingPanel,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(106.0),
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
                Text::new("合成"),
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
                    grid_template_columns: RepeatedGridTrack::px(2, CRAFTING_SLOT_SIZE),
                    grid_template_rows: RepeatedGridTrack::px(2, CRAFTING_SLOT_SIZE),
                    column_gap: Val::Px(4.0),
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|grid| {
                    for index in 0..PlayerCrafting::SLOT_COUNT {
                        spawn_empty_slot(grid, SlotKind::Container, index, &slot_theme, &ui_font);
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
                SlotKind::Container,
                PlayerCrafting::SLOT_COUNT,
                &slot_theme,
                &ui_font,
            );
        });
    });
}

pub fn crafting_visual_sync_system(
    crafting: Res<PlayerCrafting>,
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
        if slot.kind != SlotKind::Container || slot.index > PlayerCrafting::SLOT_COUNT {
            continue;
        }
        let current = if slot.index < PlayerCrafting::SLOT_COUNT {
            crafting
                .get_stack(slot.index)
                .map(|stack| (stack.item.clone(), stack.count))
                .unwrap_or((ItemId::air(), 0))
        } else {
            crafting
                .output()
                .map(|stack| (stack.item.clone(), stack.count))
                .unwrap_or((ItemId::air(), 0))
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
