use crate::game::inventory::container::world::ContainerId;
use crate::game::player::identity::PlayerId;
use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftingStationOpened {
    pub player_id: PlayerId,
    pub container_id: ContainerId,
    pub position: IVec3,
}
