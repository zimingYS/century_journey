//! 生成并同步槽位中的工具耐久条。

use bevy::prelude::*;

use super::components::{SlotDurabilityBar, SlotDurabilityFill};
use crate::content::item::ItemRegistry;
use crate::game::crafting::grid::{
    ActiveCrafting, CraftingGrid, PlayerCrafting, WorkbenchCrafting,
};
use crate::game::inventory::container::ContainerKind;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::inventory::item::stack::ItemStack;
use crate::game::inventory::slot::SlotKind;
use crate::game::inventory::state::InventoryState;
use crate::game::player::identity::LocalPlayer;
/// 在槽位内创建默认隐藏的耐久度条。
pub(super) fn spawn_durability_bar(
    parent: &mut ChildSpawnerCommands,
    kind: SlotKind,
    index: usize,
) {
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

/// 根据槽位物品实例耐久度同步耐久条宽度、颜色和可见性。
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
