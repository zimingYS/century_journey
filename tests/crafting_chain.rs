use century_journey::content::recipe::loader::load_recipe_definitions;
use century_journey::content::recipe::registry::RecipeRegistry;
use century_journey::content::tag::runtime::ItemTagIndex;
use century_journey::engine::asset::AssetManager;
use century_journey::game::crafting::grid::{PlayerCrafting, WorkbenchCrafting};
use century_journey::game::inventory::container::InventoryContainer;
use century_journey::game::inventory::item::stack::ItemStack;
use century_journey::shared::item_id::ItemId;

fn item(name: &str) -> ItemId {
    ItemId::item(format!("century_journey:{name}"))
}

fn recipes() -> RecipeRegistry {
    let mut registry = RecipeRegistry::default();
    for (id, recipe) in load_recipe_definitions(&AssetManager::default()) {
        registry.register(id, recipe);
    }
    registry
}

#[test]
fn wood_planks_workbench_and_sticks_form_the_wooden_axe_chain() {
    let recipes = recipes();
    let tags = ItemTagIndex::default();

    let mut player = PlayerCrafting::default();
    player.set_stack(0, ItemStack::single(item("wood")));
    player.refresh(&recipes, &tags);
    assert_eq!(
        player
            .output()
            .map(|stack| (stack.item.clone(), stack.count)),
        Some((item("planks"), 4))
    );

    let mut player = PlayerCrafting::default();
    player.set_stack(0, ItemStack::single(item("planks")));
    player.set_stack(2, ItemStack::single(item("planks")));
    player.refresh(&recipes, &tags);
    assert_eq!(
        player
            .output()
            .map(|stack| (stack.item.clone(), stack.count)),
        Some((item("stick"), 4))
    );

    let mut player = PlayerCrafting::default();
    for index in 0..PlayerCrafting::SLOT_COUNT {
        player.set_stack(index, ItemStack::single(item("planks")));
    }
    player.refresh(&recipes, &tags);
    assert_eq!(
        player
            .output()
            .map(|stack| (stack.item.clone(), stack.count)),
        Some((item("crafting_table"), 1))
    );

    let mut workbench = WorkbenchCrafting::default();
    for index in [0, 1, 3] {
        workbench.set_stack(index, ItemStack::single(item("planks")));
    }
    for index in [4, 7] {
        workbench.set_stack(index, ItemStack::single(item("stick")));
    }
    workbench.refresh(&recipes, &tags);
    assert_eq!(
        workbench
            .output()
            .map(|stack| (stack.item.clone(), stack.count)),
        Some((item("wooden_axe"), 1))
    );
}
