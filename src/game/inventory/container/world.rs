//! 管理工作台、箱子和熔炉等坐标绑定的世界容器实例。

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use crate::game::crafting::grid::WorkbenchCrafting;
use crate::game::inventory::container::ContainerKind;
use crate::game::inventory::container::{
    ContainerLayout, ContainerSlotRole, GameContainer, InventoryContainer,
};
use crate::game::inventory::item::stack::ItemStack;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// 在当前世界会话内稳定引用一个容器实例的 ID。
pub struct ContainerId(pub u64);

impl ContainerId {
    /// 使用持久化数值创建容器 ID。
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone)]
/// 箱子和熔炉共享的定长槽位存储实现。
pub struct StorageContainer {
    kind: ContainerKind,
    layout: ContainerLayout,
    slots: Vec<Option<ItemStack>>,
}

impl StorageContainer {
    /// 创建九列三行的标准箱子容器。
    pub fn chest() -> Self {
        Self::new(ContainerKind::Chest, ContainerLayout::new(9, 3))
    }

    /// 创建包含输入、燃料和输出槽的熔炉容器。
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
/// 世界坐标处可能存在的权威容器实例。
pub enum WorldContainer {
    /// 三乘三工作台合成容器。
    Workbench(WorkbenchCrafting),
    /// 通用箱子存储容器。
    Chest(StorageContainer),
    /// 具有专用槽位语义的熔炉容器。
    Furnace(StorageContainer),
}

impl WorldContainer {
    /// 按容器类别创建世界容器；玩家随身合成区不属于世界容器。
    pub fn new(kind: ContainerKind) -> Option<Self> {
        match kind {
            ContainerKind::Workbench => Some(Self::Workbench(WorkbenchCrafting::default())),
            ContainerKind::Chest => Some(Self::Chest(StorageContainer::chest())),
            ContainerKind::Furnace => Some(Self::Furnace(StorageContainer::furnace())),
            ContainerKind::PlayerCrafting => None,
        }
    }

    /// 当实例为工作台时返回其只读合成网格。
    pub fn workbench(&self) -> Option<&WorkbenchCrafting> {
        match self {
            Self::Workbench(workbench) => Some(workbench),
            _ => None,
        }
    }

    /// 当实例为工作台时返回其可变合成网格。
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
/// 维护世界坐标、稳定 ID 与容器实例之间的权威映射。
pub struct WorldContainers {
    next_id: u64,
    by_position: HashMap<(IVec3, ContainerKind), ContainerId>,
    containers: BTreeMap<ContainerId, WorldContainer>,
}

impl WorldContainers {
    /// 返回指定坐标和类别的容器 ID，不存在时创建对应实例。
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

    /// 按 ID 查询只读世界容器。
    pub fn get(&self, id: ContainerId) -> Option<&WorldContainer> {
        self.containers.get(&id)
    }

    /// 按 ID 查询可变世界容器。
    pub fn get_mut(&mut self, id: ContainerId) -> Option<&mut WorldContainer> {
        self.containers.get_mut(&id)
    }

    /// 按 ID 查询只读工作台合成网格。
    pub fn workbench(&self, id: ContainerId) -> Option<&WorkbenchCrafting> {
        self.get(id).and_then(WorldContainer::workbench)
    }

    /// 按 ID 查询可变工作台合成网格。
    pub fn workbench_mut(&mut self, id: ContainerId) -> Option<&mut WorkbenchCrafting> {
        self.get_mut(id).and_then(WorldContainer::workbench_mut)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/game/inventory/container/world.rs"]
mod tests;
