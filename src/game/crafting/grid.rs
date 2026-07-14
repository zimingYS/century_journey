use bevy::prelude::Resource;

use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::recipe::definition::shaped_recipe::ShapedRecipe;
use crate::content::recipe::definition::{Ingredient, RecipeResult};
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::inventory::container::InventoryContainer;
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;

#[derive(Resource, Debug, Clone)]
pub struct PlayerCrafting {
    slots: [Option<ItemStack>; Self::SLOT_COUNT],
    output: Option<ItemStack>,
}

impl Default for PlayerCrafting {
    fn default() -> Self {
        Self {
            slots: std::array::from_fn(|_| None),
            output: None,
        }
    }
}

impl PlayerCrafting {
    pub const WIDTH: usize = 2;
    pub const HEIGHT: usize = 2;
    pub const SLOT_COUNT: usize = Self::WIDTH * Self::HEIGHT;

    pub fn output(&self) -> Option<&ItemStack> {
        self.output.as_ref()
    }

    pub fn refresh(&mut self, recipes: &RecipeRegistry, tags: &ItemTagIndex) {
        self.output = find_recipe(&self.slots, recipes, tags)
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

    pub fn drain_inputs(&mut self) -> [Option<ItemStack>; Self::SLOT_COUNT] {
        self.output = None;
        std::mem::replace(&mut self.slots, std::array::from_fn(|_| None))
    }
}

impl InventoryContainer for PlayerCrafting {
    fn slot_count(&self) -> usize {
        Self::SLOT_COUNT
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

fn find_recipe(
    slots: &[Option<ItemStack>; PlayerCrafting::SLOT_COUNT],
    recipes: &RecipeRegistry,
    tags: &ItemTagIndex,
) -> Option<RecipeResult> {
    let mut entries: Vec<_> = recipes.all_recipes().collect();
    entries.sort_by_key(|(identifier, _)| *identifier);
    entries.into_iter().find_map(|(_, recipe)| {
        let result = match recipe {
            RecipeDefinition::Shaped(recipe) => {
                matches_shaped(slots, recipe, tags).then_some(recipe.result.clone())
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
    slots: &[Option<ItemStack>; PlayerCrafting::SLOT_COUNT],
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
    if height == 0 || width == 0 || height > PlayerCrafting::HEIGHT || width > PlayerCrafting::WIDTH
    {
        return false;
    }

    for mirror in [false, true] {
        for offset_y in 0..=(PlayerCrafting::HEIGHT - height) {
            for offset_x in 0..=(PlayerCrafting::WIDTH - width) {
                if shaped_at(
                    slots, recipe, tags, &rows, width, height, offset_x, offset_y, mirror,
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
    slots: &[Option<ItemStack>; PlayerCrafting::SLOT_COUNT],
    recipe: &ShapedRecipe,
    tags: &ItemTagIndex,
    rows: &[Vec<char>],
    width: usize,
    height: usize,
    offset_x: usize,
    offset_y: usize,
    mirror: bool,
) -> bool {
    for grid_y in 0..PlayerCrafting::HEIGHT {
        for grid_x in 0..PlayerCrafting::WIDTH {
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
            let slot = slots[grid_y * PlayerCrafting::WIDTH + grid_x].as_ref();
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
    slots: &[Option<ItemStack>; PlayerCrafting::SLOT_COUNT],
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
mod tests {
    use super::*;
    use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
    use crate::content::recipe::definition::shaped_recipe::ShapedRecipe;
    use crate::content::recipe::definition::shapeless_recipe::ShapelessRecipe;
    use crate::shared::identifier::Identifier;

    fn item(name: &str) -> ItemId {
        ItemId::item(format!("century_journey:{name}"))
    }

    #[test]
    fn shaped_recipe_matches_offset_and_consumes_inputs() {
        let mut recipes = RecipeRegistry::default();
        recipes.register(
            Identifier::parse("test:stick").unwrap(),
            RecipeDefinition::Shaped(ShapedRecipe {
                pattern: vec!["A".into(), "A".into()],
                key: [('A', Ingredient::Item { item: item("wood") })]
                    .into_iter()
                    .collect(),
                result: RecipeResult {
                    item: item("stick"),
                    count: 4,
                },
            }),
        );
        let mut crafting = PlayerCrafting::default();
        crafting.set_stack(1, ItemStack::new(item("wood"), 3));
        crafting.set_stack(3, ItemStack::new(item("wood"), 2));
        crafting.refresh(&recipes, &ItemTagIndex::default());
        assert_eq!(crafting.output().map(|stack| stack.count), Some(4));
        crafting.consume_recipe();
        assert_eq!(crafting.get_stack(1).map(|stack| stack.count), Some(2));
        assert_eq!(crafting.get_stack(3).map(|stack| stack.count), Some(1));
    }

    #[test]
    fn shapeless_recipe_accepts_reordered_inputs() {
        let mut recipes = RecipeRegistry::default();
        recipes.register(
            Identifier::parse("test:pair").unwrap(),
            RecipeDefinition::Shapeless(ShapelessRecipe {
                ingredients: vec![
                    Ingredient::Item { item: item("wood") },
                    Ingredient::Item {
                        item: item("stick"),
                    },
                ],
                result: RecipeResult {
                    item: item("axe"),
                    count: 1,
                },
            }),
        );
        let mut crafting = PlayerCrafting::default();
        crafting.set_stack(0, ItemStack::single(item("stick")));
        crafting.set_stack(2, ItemStack::single(item("wood")));
        crafting.refresh(&recipes, &ItemTagIndex::default());
        assert_eq!(
            crafting.output().map(|stack| stack.item.clone()),
            Some(item("axe"))
        );
    }

    #[test]
    fn shaped_recipe_rejects_extra_items() {
        let mut recipes = RecipeRegistry::default();
        recipes.register(
            Identifier::parse("test:stick").unwrap(),
            RecipeDefinition::Shaped(ShapedRecipe {
                pattern: vec!["A".into(), "A".into()],
                key: [('A', Ingredient::Item { item: item("wood") })]
                    .into_iter()
                    .collect(),
                result: RecipeResult {
                    item: item("stick"),
                    count: 1,
                },
            }),
        );
        let mut crafting = PlayerCrafting::default();
        crafting.set_stack(0, ItemStack::single(item("wood")));
        crafting.set_stack(1, ItemStack::single(item("wood")));
        crafting.set_stack(2, ItemStack::single(item("wood")));
        crafting.refresh(&recipes, &ItemTagIndex::default());
        assert!(crafting.output().is_none());
    }

    #[test]
    fn stage_seven_collected_sticks_craft_bootstrap_axe() {
        let mut recipes = RecipeRegistry::default();
        recipes.register(
            Identifier::parse("century_journey:wooden_axe_from_sticks").unwrap(),
            RecipeDefinition::Shaped(ShapedRecipe {
                pattern: vec!["AA".into(), " A".into()],
                key: [(
                    'A',
                    Ingredient::Item {
                        item: item("stick"),
                    },
                )]
                .into_iter()
                .collect(),
                result: RecipeResult {
                    item: item("wooden_axe"),
                    count: 1,
                },
            }),
        );
        let mut crafting = PlayerCrafting::default();
        crafting.set_stack(0, ItemStack::single(item("stick")));
        crafting.set_stack(1, ItemStack::single(item("stick")));
        crafting.set_stack(3, ItemStack::single(item("stick")));

        crafting.refresh(&recipes, &ItemTagIndex::default());

        assert_eq!(
            crafting.output().map(|stack| stack.item.clone()),
            Some(item("wooden_axe"))
        );
        crafting.consume_recipe();
        assert!(crafting.slots.iter().all(Option::is_none));
    }
}
