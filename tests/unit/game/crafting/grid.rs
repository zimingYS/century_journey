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
fn wooden_axe_requires_the_workbench_three_by_three_grid() {
    let mut recipes = RecipeRegistry::default();
    recipes.register(
        Identifier::parse("century_journey:wooden_axe").unwrap(),
        RecipeDefinition::Shaped(ShapedRecipe {
            pattern: vec!["PP ".into(), "PS ".into(), " S ".into()],
            key: [
                (
                    'P',
                    Ingredient::Item {
                        item: item("planks"),
                    },
                ),
                (
                    'S',
                    Ingredient::Item {
                        item: item("stick"),
                    },
                ),
            ]
            .into_iter()
            .collect(),
            result: RecipeResult {
                item: item("wooden_axe"),
                count: 1,
            },
        }),
    );
    let mut player = PlayerCrafting::default();
    player.refresh(&recipes, &ItemTagIndex::default());
    assert!(player.output().is_none());

    let mut crafting = WorkbenchCrafting::default();
    for index in [0, 1, 3] {
        crafting.set_stack(index, ItemStack::single(item("planks")));
    }
    for index in [4, 7] {
        crafting.set_stack(index, ItemStack::single(item("stick")));
    }
    crafting.refresh(&recipes, &ItemTagIndex::default());
    assert_eq!(
        crafting.output().map(|stack| stack.item.clone()),
        Some(item("wooden_axe"))
    );
    crafting.consume_recipe();
    assert!((0..crafting.slot_count()).all(|index| crafting.get_stack(index).is_none()));
}

#[test]
fn matcher_accepts_grids_larger_than_the_current_workbench() {
    let mut recipes = RecipeRegistry::default();
    recipes.register(
        Identifier::parse("test:wide").unwrap(),
        RecipeDefinition::Shaped(ShapedRecipe {
            pattern: vec!["AAAA".into()],
            key: [(
                'A',
                Ingredient::Item {
                    item: item("planks"),
                },
            )]
            .into_iter()
            .collect(),
            result: RecipeResult {
                item: item("wide_result"),
                count: 1,
            },
        }),
    );
    let mut grid = CraftingGrid::new(4, 3);
    for index in 4..8 {
        grid.set_stack(index, ItemStack::single(item("planks")));
    }

    grid.refresh(&recipes, &ItemTagIndex::default());

    assert_eq!(
        grid.output().map(|stack| stack.item.clone()),
        Some(item("wide_result"))
    );
}
