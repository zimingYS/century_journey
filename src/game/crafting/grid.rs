use bevy::prelude::{Component, IVec3};

use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::recipe::definition::shaped_recipe::ShapedRecipe;
use crate::content::recipe::definition::{Ingredient, RecipeResult};
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::inventory::container::world::ContainerId;
use crate::game::inventory::container::{
    ContainerLayout, ContainerSlotRole, GameContainer, InventoryContainer,
};
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;
use crate::shared::ui_types::ContainerKind;

#[derive(Debug, Clone)]
pub struct CraftingGrid {
    width: usize,
    height: usize,
    slots: Vec<Option<ItemStack>>,
    output: Option<ItemStack>,
}

impl CraftingGrid {
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

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn output(&self) -> Option<&ItemStack> {
        self.output.as_ref()
    }

    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.output = find_recipe(&self.slots, self.width, self.height, recipes, tags)
            .map(|result| ItemStack::new(result.item, result.count));
    }

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
pub struct PlayerCrafting(CraftingGrid);

impl PlayerCrafting {
    pub const WIDTH: usize = 2;
    pub const HEIGHT: usize = 2;
    pub const SLOT_COUNT: usize = Self::WIDTH * Self::HEIGHT;

    pub fn grid(&self) -> &CraftingGrid {
        &self.0
    }

    pub fn grid_mut(&mut self) -> &mut CraftingGrid {
        &mut self.0
    }

    pub fn output(&self) -> Option<&ItemStack> {
        self.0.output()
    }

    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.0.refresh(recipes, tags);
    }

    pub fn consume_recipe(&mut self) {
        self.0.consume_recipe();
    }

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
pub struct WorkbenchCrafting(CraftingGrid);

impl WorkbenchCrafting {
    pub const WIDTH: usize = 3;
    pub const HEIGHT: usize = 3;
    pub const SLOT_COUNT: usize = Self::WIDTH * Self::HEIGHT;

    pub fn grid(&self) -> &CraftingGrid {
        &self.0
    }

    pub fn grid_mut(&mut self) -> &mut CraftingGrid {
        &mut self.0
    }

    pub fn output(&self) -> Option<&ItemStack> {
        self.0.output()
    }

    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.0.refresh(recipes, tags);
    }

    pub fn consume_recipe(&mut self) {
        self.0.consume_recipe();
    }

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
pub struct ActiveCrafting {
    pub kind: ContainerKind,
    pub station_position: Option<IVec3>,
    pub container_id: Option<ContainerId>,
    pub was_opened: bool,
}

impl Default for ActiveCrafting {
    fn default() -> Self {
        Self::player()
    }
}

impl ActiveCrafting {
    pub const fn player() -> Self {
        Self {
            kind: ContainerKind::PlayerCrafting,
            station_position: None,
            container_id: None,
            was_opened: false,
        }
    }

    pub const fn workbench(position: IVec3, container_id: ContainerId) -> Self {
        Self {
            kind: ContainerKind::Workbench,
            station_position: Some(position),
            container_id: Some(container_id),
            was_opened: false,
        }
    }
}

fn find_recipe(
    slots: &[Option<ItemStack>],
    grid_width: usize,
    grid_height: usize,
    recipes: &RecipeRegistry,
    tags: &ItemTagIndex,
) -> Option<RecipeResult> {
    let mut entries: Vec<_> = recipes.all_recipes().collect();
    entries.sort_by_key(|(identifier, _)| *identifier);
    entries.into_iter().find_map(|(_, recipe)| {
        let result = match recipe {
            RecipeDefinition::Shaped(recipe) => {
                matches_shaped(slots, grid_width, grid_height, recipe, tags)
                    .then_some(recipe.result.clone())
            }
            RecipeDefinition::Shapeless(recipe) => {
                matches_shapeless(slots, &recipe.ingredients, tags).then_some(recipe.result.clone())
            }
        }?;
        (result.count > 0 && !result.item.is_air()).then_some(result)
    })
}

fn ingredient_matches(ingredient: &Ingredient, id: &ItemId, tags: &ItemTagIndex) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { tag } => tags.contains(tag, id),
    }
}

fn matches_shaped(
    slots: &[Option<ItemStack>],
    grid_width: usize,
    grid_height: usize,
    recipe: &ShapedRecipe,
    tags: &ItemTagIndex,
) -> bool {
    let rows: Vec<Vec<char>> = recipe
        .pattern
        .iter()
        .map(|row| row.chars().collect())
        .collect();
    let height = rows.len();
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if height == 0 || width == 0 || height > grid_height || width > grid_width {
        return false;
    }

    for mirror in [false, true] {
        for offset_y in 0..=(grid_height - height) {
            for offset_x in 0..=(grid_width - width) {
                if shaped_at(
                    slots,
                    grid_width,
                    grid_height,
                    recipe,
                    tags,
                    &rows,
                    width,
                    height,
                    offset_x,
                    offset_y,
                    mirror,
                ) {
                    return true;
                }
            }
        }
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn shaped_at(
    slots: &[Option<ItemStack>],
    grid_width: usize,
    grid_height: usize,
    recipe: &ShapedRecipe,
    tags: &ItemTagIndex,
    rows: &[Vec<char>],
    width: usize,
    height: usize,
    offset_x: usize,
    offset_y: usize,
    mirror: bool,
) -> bool {
    for grid_y in 0..grid_height {
        for grid_x in 0..grid_width {
            let key = grid_x
                .checked_sub(offset_x)
                .zip(grid_y.checked_sub(offset_y))
                .filter(|(x, y)| *x < width && *y < height)
                .and_then(|(x, y)| {
                    let pattern_x = if mirror { width - 1 - x } else { x };
                    rows.get(y).and_then(|row| row.get(pattern_x))
                })
                .copied()
                .unwrap_or(' ');
            let slot = slots[grid_y * grid_width + grid_x].as_ref();
            if key == ' ' {
                if slot.is_some_and(|stack| !stack.is_empty()) {
                    return false;
                }
                continue;
            }
            let Some(ingredient) = recipe.key.get(&key) else {
                return false;
            };
            let Some(stack) = slot else {
                return false;
            };
            if !ingredient_matches(ingredient, &stack.item, tags) {
                return false;
            }
        }
    }
    true
}

fn matches_shapeless(
    slots: &[Option<ItemStack>],
    ingredients: &[Ingredient],
    tags: &ItemTagIndex,
) -> bool {
    let inputs: Vec<&ItemId> = slots
        .iter()
        .filter_map(Option::as_ref)
        .filter(|stack| !stack.is_empty())
        .map(|stack| &stack.item)
        .collect();
    if inputs.len() != ingredients.len() {
        return false;
    }
    let mut used = vec![false; inputs.len()];
    match_ingredients(ingredients, &inputs, tags, &mut used, 0)
}

fn match_ingredients(
    ingredients: &[Ingredient],
    inputs: &[&ItemId],
    tags: &ItemTagIndex,
    used: &mut [bool],
    ingredient_index: usize,
) -> bool {
    if ingredient_index == ingredients.len() {
        return true;
    }
    for input_index in 0..inputs.len() {
        if !used[input_index]
            && ingredient_matches(&ingredients[ingredient_index], inputs[input_index], tags)
        {
            used[input_index] = true;
            if match_ingredients(ingredients, inputs, tags, used, ingredient_index + 1) {
                return true;
            }
            used[input_index] = false;
        }
    }
    false
}

#[cfg(test)]
#[path = "../../../tests/unit/game/crafting/grid.rs"]
mod tests;
