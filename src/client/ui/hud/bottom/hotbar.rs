use crate::client::renderer::item_model::ItemModelRenderAssets;
use crate::client::renderer::tex_atlas::BlockRenderAssets;
use crate::client::ui::hud::bottom::BottomHud;
use crate::client::ui::resources::ui_font::UiFont;
use crate::client::ui::theme::ui_theme::UiTheme;
use crate::client::ui::widgets::slot::{
    InventorySlot, SlotKind, SlotVisual, spawn_display_only_slot, sync_slot_icon,
};
use crate::content::block::registry::BlockRegistry;
use crate::content::item::registry::registry::ItemRegistry;
use crate::content::item::texture::registry::ItemTextureRegistry;
use crate::game::inventory::container::hotbar::HOTBAR_SIZE;
use crate::game::inventory::state::InventoryState;
use crate::game::player::action::{PlayerAction, PlayerActionState};
use crate::shared::item_id::ItemId;
use bevy::prelude::*;

/// HUD快捷栏(物品栏)
#[derive(Component)]
pub struct Hotbar;

/// HUD快捷栏(物品栏)外的高亮选择框
#[derive(Component)]
pub struct HotbarSelector;

/// 生成HUD快捷栏
pub fn spawn_hotbar_ui_system(
    mut commands: Commands,
    theme: Res<UiTheme>,
    ui_font: Res<UiFont>,
    bottom_hud: Query<Entity, With<BottomHud>>,
) {
    let Ok(bottom_hud_entity) = bottom_hud.single() else {
        log::error!("BOTTOM HUD NOT FOUND — cannot spawn hotbar");
        return;
    };

    commands.entity(bottom_hud_entity).with_children(|root| {
        root.spawn((
            Hotbar,
            Name::new("HudHotbar"),
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(theme.slot_gap),
                padding: UiRect::all(Val::Px(4.0)),
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect {
                    bottom: Val::Px(20.0),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(theme.hotbar_bg),
            BorderColor::all(theme.border_default),
        ))
        .with_children(|parent| {
            for index in 0..HOTBAR_SIZE {
                spawn_display_only_slot(parent, SlotKind::Hotbar, index, &theme, &ui_font);
            }
        });
    });
}

/// HUD快捷栏视觉同步
pub fn hud_hotbar_visual_sync_system(
    state: Res<InventoryState>,
    block_registry: Option<Res<BlockRegistry>>,
    block_render_assets: Option<Res<BlockRenderAssets>>,
    item_model_assets: Res<ItemModelRenderAssets>,
    mut commands: Commands,
    slot_query: Query<(Entity, &InventorySlot)>,
    mut slot_visual_query: Query<&mut SlotVisual>,
    children_query: Query<&Children>,
    mut border_query: Query<(&InventorySlot, &mut BorderColor)>,
    theme: Res<UiTheme>,
    mut last_hotbar: Local<Option<(Vec<(ItemId, u32)>, u64)>>,
    mut last_active: Local<usize>,
    mut was_opened: Local<bool>,
    item_registry: Option<Res<ItemRegistry>>,
    item_texture_registry: Option<Res<ItemTextureRegistry>>,
) {
    let Some(reg) = block_registry.as_ref() else {
        return;
    };
    let Some(render_assets) = block_render_assets.as_ref() else {
        return;
    };

    // 背包打开时强制重置缓存（确保 HUD hotbar 与数据同步）
    if state.opened && !*was_opened {
        *last_hotbar = None;
    }
    *was_opened = state.opened;

    // 构建当前快照（包含数量）
    let current: Vec<(ItemId, u32)> = (0..HOTBAR_SIZE)
        .map(|i| {
            state
                .hotbar
                .get_stack(i)
                .map(|s| (s.item.clone(), s.count))
                .unwrap_or((ItemId::air(), 0))
        })
        .collect();

    // 图标同步 — 物品、数量或 3D 图标缓存版本变化时执行
    let revision = item_model_assets.revision();
    let force = last_hotbar.is_none();
    let revision_changed = last_hotbar
        .as_ref()
        .is_some_and(|(_, cached_revision)| *cached_revision != revision);
    let changed = force
        || last_hotbar.as_ref().is_none_or(|(items, cached_revision)| {
            items != &current || *cached_revision != revision
        });
    if changed {
        *last_hotbar = Some((current.clone(), revision));

        for (entity, slot) in &slot_query {
            if slot.kind != SlotKind::Hotbar {
                continue;
            }
            let (item, count) = current
                .get(slot.index)
                .cloned()
                .unwrap_or((ItemId::air(), 0));

            if let Ok(mut visual) = slot_visual_query.get_mut(entity)
                && (force || revision_changed || visual.item != item || visual.count != count)
            {
                sync_slot_icon(
                    &mut commands,
                    entity,
                    &item,
                    count,
                    reg,
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

    // 边框高亮 — 仅在 active_index 变化时更新，避免覆盖 hover 高亮
    if *last_active != state.hotbar.active_index {
        *last_active = state.hotbar.active_index;
        for (slot, mut border) in &mut border_query {
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
}

/// 数字键/滚轮切换快捷栏
pub fn handle_hotbar_switch_system(
    actions: Res<PlayerActionState>,
    mut state: ResMut<InventoryState>,
) {
    if !actions.pressed(PlayerAction::Hotbar1)
        && !actions.pressed(PlayerAction::Hotbar2)
        && !actions.pressed(PlayerAction::Hotbar3)
        && !actions.pressed(PlayerAction::Hotbar4)
        && !actions.pressed(PlayerAction::Hotbar5)
        && !actions.pressed(PlayerAction::Hotbar6)
        && !actions.pressed(PlayerAction::Hotbar7)
        && !actions.pressed(PlayerAction::Hotbar8)
        && !actions.pressed(PlayerAction::Hotbar9)
        && !actions.pressed(PlayerAction::HotbarPrevious)
        && !actions.pressed(PlayerAction::HotbarNext)
    {
        return;
    }

    let hotbar_actions = [
        PlayerAction::Hotbar1,
        PlayerAction::Hotbar2,
        PlayerAction::Hotbar3,
        PlayerAction::Hotbar4,
        PlayerAction::Hotbar5,
        PlayerAction::Hotbar6,
        PlayerAction::Hotbar7,
        PlayerAction::Hotbar8,
        PlayerAction::Hotbar9,
    ];
    for (index, action) in hotbar_actions.into_iter().enumerate() {
        if actions.just_pressed(action) {
            state.hotbar.active_index = index;
            break;
        }
    }

    if actions.just_pressed(PlayerAction::HotbarPrevious) {
        state.hotbar.select_prev();
    }
    if actions.just_pressed(PlayerAction::HotbarNext) {
        state.hotbar.select_next();
    }
}
