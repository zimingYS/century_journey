use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use crate::game::crafting::grid::WorkbenchCrafting;
use crate::game::inventory::container::{
    ContainerLayout, ContainerSlotRole, GameContainer, InventoryContainer,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::ui_types::ContainerKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContainerId(pub u64);

impl ContainerId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
pub struct StorageContainer {
    kind: ContainerKind,
    layout: ContainerLayout,
    slots: Vec<Option<ItemStack>>,
}

impl StorageContainer {
    pub fn chest() -> Self {
        Self::new(ContainerKind::Chest, ContainerLayout::new(9, 3))
    }

    pub fn furnace() -> Self {
        Self::new(ContainerKind::Furnace, ContainerLayout::new(1, 3))
    }

    fn new(kind: ContainerKind, layout: ContainerLayout) -> Self {
        Self {
            kind,
            layout,
            slots: vec![None; layout.slot_count()],
        }
    }
}

impl InventoryContainer for StorageContainer {
    fn slot_count(&self) -> usize {
        self.slots.len()
    }

    fn get_stack(&self, index: usize) -> Option<&ItemStack> {
        self.slots.get(index).and_then(Option::as_ref)
    }

    fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        self.slots.get_mut(index).and_then(Option::as_mut)
    }

    fn set_stack(&mut self, index: usize, stack: ItemStack) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = (!stack.is_empty()).then_some(stack);
        }
    }
}

impl GameContainer for StorageContainer {
    fn kind(&self) -> ContainerKind {
        self.kind
    }

    fn layout(&self) -> ContainerLayout {
        self.layout
    }

    fn slot_role(&self, index: usize) -> ContainerSlotRole {
        match self.kind {
            ContainerKind::Furnace if index == 0 => ContainerSlotRole::Input,
            ContainerKind::Furnace if index == 1 => ContainerSlotRole::Fuel,
            ContainerKind::Furnace if index == 2 => ContainerSlotRole::Output,
            _ => ContainerSlotRole::Storage,
        }
    }
}

#[derive(Debug, Clone)]
pub enum WorldContainer {
    Workbench(WorkbenchCrafting),
    Chest(StorageContainer),
    Furnace(StorageContainer),
}

impl WorldContainer {
    pub fn new(kind: ContainerKind) -> Option<Self> {
        match kind {
            ContainerKind::Workbench => Some(Self::Workbench(WorkbenchCrafting::default())),
            ContainerKind::Chest => Some(Self::Chest(StorageContainer::chest())),
            ContainerKind::Furnace => Some(Self::Furnace(StorageContainer::furnace())),
            ContainerKind::PlayerCrafting => None,
        }
    }

    pub fn workbench(&self) -> Option<&WorkbenchCrafting> {
        match self {
            Self::Workbench(workbench) => Some(workbench),
            _ => None,
        }
    }

    pub fn workbench_mut(&mut self) -> Option<&mut WorkbenchCrafting> {
        match self {
            Self::Workbench(workbench) => Some(workbench),
            _ => None,
        }
    }
}

impl InventoryContainer for WorldContainer {
    fn slot_count(&self) -> usize {
        match self {
            Self::Workbench(value) => value.slot_count(),
            Self::Chest(value) | Self::Furnace(value) => value.slot_count(),
        }
    }

    fn get_stack(&self, index: usize) -> Option<&ItemStack> {
        match self {
            Self::Workbench(value) => value.get_stack(index),
            Self::Chest(value) | Self::Furnace(value) => value.get_stack(index),
        }
    }

    fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
        match self {
            Self::Workbench(value) => value.get_stack_mut(index),
            Self::Chest(value) | Self::Furnace(value) => value.get_stack_mut(index),
        }
    }

    fn set_stack(&mut self, index: usize, stack: ItemStack) {
        match self {
            Self::Workbench(value) => value.set_stack(index, stack),
            Self::Chest(value) | Self::Furnace(value) => value.set_stack(index, stack),
        }
    }
}

impl GameContainer for WorldContainer {
    fn kind(&self) -> ContainerKind {
        match self {
            Self::Workbench(_) => ContainerKind::Workbench,
            Self::Chest(_) => ContainerKind::Chest,
            Self::Furnace(_) => ContainerKind::Furnace,
        }
    }

    fn layout(&self) -> ContainerLayout {
        match self {
            Self::Workbench(value) => value.layout(),
            Self::Chest(value) | Self::Furnace(value) => value.layout(),
        }
    }

    fn slot_role(&self, index: usize) -> ContainerSlotRole {
        match self {
            Self::Workbench(value) => value.slot_role(index),
            Self::Chest(value) | Self::Furnace(value) => value.slot_role(index),
        }
    }
}

#[derive(Resource, Debug, Default)]
pub struct WorldContainers {
    next_id: u64,
    by_position: HashMap<(IVec3, ContainerKind), ContainerId>,
    containers: BTreeMap<ContainerId, WorldContainer>,
}

impl WorldContainers {
    pub fn ensure_at(&mut self, position: IVec3, kind: ContainerKind) -> Option<ContainerId> {
        if let Some(id) = self.by_position.get(&(position, kind)).copied() {
            return Some(id);
        }
        let container = WorldContainer::new(kind)?;
        self.next_id = self.next_id.saturating_add(1);
        let id = ContainerId(self.next_id);
        self.by_position.insert((position, kind), id);
        self.containers.insert(id, container);
        Some(id)
    }

    pub fn get(&self, id: ContainerId) -> Option<&WorldContainer> {
        self.containers.get(&id)
    }

    pub fn get_mut(&mut self, id: ContainerId) -> Option<&mut WorldContainer> {
        self.containers.get_mut(&id)
    }

    pub fn workbench(&self, id: ContainerId) -> Option<&WorkbenchCrafting> {
        self.get(id).and_then(WorldContainer::workbench)
    }

    pub fn workbench_mut(&mut self, id: ContainerId) -> Option<&mut WorkbenchCrafting> {
        self.get_mut(id).and_then(WorldContainer::workbench_mut)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workbench_chest_and_furnace_receive_distinct_container_ids() {
        let mut containers = WorldContainers::default();
        let workbench = containers
            .ensure_at(IVec3::ZERO, ContainerKind::Workbench)
            .unwrap();
        let chest = containers
            .ensure_at(IVec3::X, ContainerKind::Chest)
            .unwrap();
        let furnace = containers
            .ensure_at(IVec3::Z, ContainerKind::Furnace)
            .unwrap();

        assert_ne!(workbench, chest);
        assert_ne!(workbench, furnace);
        assert_ne!(chest, furnace);
        assert_eq!(
            containers.get(workbench).unwrap().kind(),
            ContainerKind::Workbench
        );
        assert_eq!(containers.get(chest).unwrap().kind(), ContainerKind::Chest);
        assert_eq!(
            containers.get(furnace).unwrap().kind(),
            ContainerKind::Furnace
        );
        assert_eq!(
            containers.get(furnace).unwrap().slot_role(1),
            ContainerSlotRole::Fuel
        );
        assert_eq!(
            containers.get(furnace).unwrap().slot_role(2),
            ContainerSlotRole::Output
        );
    }
}
