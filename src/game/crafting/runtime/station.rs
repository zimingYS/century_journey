use crate::content::block::event::BlockInteractEvent;
use crate::content::block::registry::BlockRegistry;
use crate::game::crafting::events::CraftingStationOpened;
use crate::game::crafting::grid::ActiveCrafting;
use crate::game::inventory::container::world::WorldContainers;
use crate::game::player::identity::PlayerId;
use crate::shared::ui_types::ContainerKind;
use bevy::prelude::{MessageReader, MessageWriter, Query, Res, ResMut};

pub fn open_workbench_system(
    mut interactions: MessageReader<BlockInteractEvent>,
    registry: Option<Res<BlockRegistry>>,
    mut players: Query<(&PlayerId, &mut ActiveCrafting)>,
    mut containers: ResMut<WorldContainers>,
    mut opened: MessageWriter<CraftingStationOpened>,
) {
    let Some(registry) = registry else { return };
    for event in interactions.read() {
        let is_workbench = registry
            .get_identifier_by_id(event.block_id)
            .is_some_and(|identifier| identifier == "century_journey:crafting_table");
        if !is_workbench {
            continue;
        }
        let Some(interactor) = event.interactor else {
            continue;
        };
        let Ok((player_id, mut active)) = players.get_mut(interactor) else {
            continue;
        };
        let Some(container_id) = containers.ensure_at(event.world_pos, ContainerKind::Workbench)
        else {
            continue;
        };
        *active = ActiveCrafting::workbench(event.world_pos, container_id);
        opened.write(CraftingStationOpened {
            player_id: *player_id,
            container_id,
            position: event.world_pos,
        });
    }
}
