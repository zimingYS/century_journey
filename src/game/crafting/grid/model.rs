//! 合成网格、玩家合成区与工作台会话的数据模型。

use bevy::prelude::{Component, IVec3};

use super::matching::find_recipe;
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::inventory::container::ContainerKind;
use crate::game::inventory::container::world::ContainerId;
use crate::game::inventory::container::{
    ContainerLayout, ContainerSlotRole, GameContainer, InventoryContainer,
};
use crate::game::inventory::item::stack::ItemStack;

#[derive(Debug, Clone)]
/// 保存矩形配方输入槽及其派生输出的通用合成网格。
pub struct CraftingGrid {
    width: usize,
    height: usize,
    slots: Vec<Option<ItemStack>>,
    output: Option<ItemStack>,
}

impl CraftingGrid {
    /// 创建尺寸固定且输入为空的合成网格。
    pub fn new(width: usize, height: usize) -> Self {
        assert!(
            width > 0 && height > 0,
            "crafting grid dimensions must be positive"
        );
        let slot_count = width
            .checked_mul(height)
            .expect("crafting grid dimensions overflowed");
        Self {
            width,
            height,
            slots: vec![None; slot_count],
            output: None,
        }
    }

    /// 返回合成网格宽度。
    pub fn width(&self) -> usize {
        self.width
    }

    /// 返回合成网格高度。
    pub fn height(&self) -> usize {
        self.height
    }

    /// 返回最近一次配方匹配得到的只读输出。
    pub fn output(&self) -> Option<&ItemStack> {
        self.output.as_ref()
    }

    /// 根据当前输入重新匹配配方并刷新派生输出。
    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.output = find_recipe(&self.slots, self.width, self.height, recipes, tags)
            .map(|result| ItemStack::new(result.item, result.count));
    }

    /// 从每个非空输入槽消耗一个物品，并清除旧输出。
    pub fn consume_recipe(&mut self) {
        for slot in &mut self.slots {
            let Some(stack) = slot else { continue };
            stack.count = stack.count.saturating_sub(1);
            if stack.is_empty() {
                *slot = None;
            }
        }
        self.output = None;
    }

    /// 取出全部输入并把网格恢复为空状态。
    pub fn drain_inputs(&mut self) -> Vec<Option<ItemStack>> {
        self.output = None;
        std::mem::replace(&mut self.slots, vec![None; self.width * self.height])
    }
}

impl InventoryContainer for CraftingGrid {
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

#[derive(Component, Debug, Clone)]
/// 玩家随身携带的二乘二合成网格组件。
pub struct PlayerCrafting(CraftingGrid);

impl PlayerCrafting {
    /// 玩家合成网格的列数。
    pub const WIDTH: usize = 2;
    /// 玩家合成网格的行数。
    pub const HEIGHT: usize = 2;
    /// 玩家合成网格的输入槽总数。
    pub const SLOT_COUNT: usize = Self::WIDTH * Self::HEIGHT;

    /// 返回底层通用合成网格。
    pub fn grid(&self) -> &CraftingGrid {
        &self.0
    }

    /// 返回底层通用合成网格的可变引用。
    pub fn grid_mut(&mut self) -> &mut CraftingGrid {
        &mut self.0
    }

    /// 返回当前匹配输出。
    pub fn output(&self) -> Option<&ItemStack> {
        self.0.output()
    }

    /// 根据玩家输入重新匹配配方。
    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.0.refresh(recipes, tags);
    }

    /// 消耗一次当前配方所需的输入。
    pub fn consume_recipe(&mut self) {
        self.0.consume_recipe();
    }

    /// 取出玩家合成区中的全部输入。
    pub fn drain_inputs(&mut self) -> Vec<Option<ItemStack>> {
        self.0.drain_inputs()
    }
}

impl Default for PlayerCrafting {
    fn default() -> Self {
        Self(CraftingGrid::new(Self::WIDTH, Self::HEIGHT))
    }
}

#[derive(Debug, Clone)]
/// 世界工作台持有的三乘三合成网格。
pub struct WorkbenchCrafting(CraftingGrid);

impl WorkbenchCrafting {
    /// 工作台合成网格的列数。
    pub const WIDTH: usize = 3;
    /// 工作台合成网格的行数。
    pub const HEIGHT: usize = 3;
    /// 工作台合成网格的输入槽总数。
    pub const SLOT_COUNT: usize = Self::WIDTH * Self::HEIGHT;

    /// 返回底层通用合成网格。
    pub fn grid(&self) -> &CraftingGrid {
        &self.0
    }

    /// 返回底层通用合成网格的可变引用。
    pub fn grid_mut(&mut self) -> &mut CraftingGrid {
        &mut self.0
    }

    /// 返回当前匹配输出。
    pub fn output(&self) -> Option<&ItemStack> {
        self.0.output()
    }

    /// 根据工作台输入重新匹配配方。
    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.0.refresh(recipes, tags);
    }

    /// 消耗一次当前配方所需的输入。
    pub fn consume_recipe(&mut self) {
        self.0.consume_recipe();
    }

    /// 取出工作台合成区中的全部输入。
    pub fn drain_inputs(&mut self) -> Vec<Option<ItemStack>> {
        self.0.drain_inputs()
    }
}

impl Default for WorkbenchCrafting {
    fn default() -> Self {
        Self(CraftingGrid::new(Self::WIDTH, Self::HEIGHT))
    }
}

macro_rules! impl_container_wrapper {
    ($type:ty, $kind:expr, $width:expr, $height:expr) => {
        impl InventoryContainer for $type {
            fn slot_count(&self) -> usize {
                self.0.slot_count()
            }

            fn get_stack(&self, index: usize) -> Option<&ItemStack> {
                self.0.get_stack(index)
            }

            fn get_stack_mut(&mut self, index: usize) -> Option<&mut ItemStack> {
                self.0.get_stack_mut(index)
            }

            fn set_stack(&mut self, index: usize, stack: ItemStack) {
                self.0.set_stack(index, stack);
            }
        }

        impl GameContainer for $type {
            fn kind(&self) -> ContainerKind {
                $kind
            }

            fn layout(&self) -> ContainerLayout {
                ContainerLayout::new($width, $height)
            }

            fn slot_role(&self, _index: usize) -> ContainerSlotRole {
                ContainerSlotRole::Input
            }
        }
    };
}

impl_container_wrapper!(
    PlayerCrafting,
    ContainerKind::PlayerCrafting,
    PlayerCrafting::WIDTH,
    PlayerCrafting::HEIGHT
);
impl_container_wrapper!(
    WorkbenchCrafting,
    ContainerKind::Workbench,
    WorkbenchCrafting::WIDTH,
    WorkbenchCrafting::HEIGHT
);

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
/// 描述玩家当前打开的合成容器及其前一帧生命周期状态。
pub struct ActiveCrafting {
    /// 当前合成容器类别。
    pub kind: ContainerKind,
    /// 世界工作台坐标；随身合成时为空。
    pub station_position: Option<IVec3>,
    /// 世界容器 ID；随身合成时为空。
    pub container_id: Option<ContainerId>,
    /// 上一次生命周期检查时物品栏是否打开。
    pub was_opened: bool,
}

impl Default for ActiveCrafting {
    fn default() -> Self {
        Self::player()
    }
}

impl ActiveCrafting {
    /// 创建默认的玩家随身合成会话。
    pub const fn player() -> Self {
        Self {
            kind: ContainerKind::PlayerCrafting,
            station_position: None,
            container_id: None,
            was_opened: false,
        }
    }

    /// 创建绑定到指定世界工作台的合成会话。
    pub const fn workbench(position: IVec3, container_id: ContainerId) -> Self {
        Self {
            kind: ContainerKind::Workbench,
            station_position: Some(position),
            container_id: Some(container_id),
            was_opened: false,
        }
    }
}
