//! 在矩形合成网格中匹配有序与无序配方。
//!
//! 匹配过程不修改网格；调用方只有在输出被接收后才负责消耗输入。

use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::recipe::definition::shaped_recipe::ShapedRecipe;
use crate::content::recipe::definition::{Ingredient, RecipeResult};
use crate::content::recipe::registry::RecipeRegistry;
use crate::content::tag::runtime::ItemTagIndex;
use crate::game::inventory::item::stack::ItemStack;
use crate::shared::item_id::ItemId;

/// 按稳定标识符顺序返回首个匹配且结果有效的配方。
///
/// 注册表的内部遍历顺序不构成确定性契约，因此必须先排序，确保相同输入在不同运行中
/// 选择同一个配方。
pub(super) fn find_recipe(
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

/// 判断单个物品是否满足直接物品或标签原料约束。
fn ingredient_matches(ingredient: &Ingredient, id: &ItemId, tags: &ItemTagIndex) -> bool {
    match ingredient {
        Ingredient::Item { item } => item == id,
        Ingredient::Tag { tag } => tags.contains(tag, id),
    }
}

/// 在所有合法偏移和水平镜像中查找有序配方。
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

    // 先检查原图，再检查水平镜像；偏移按行、列递增，保持原有稳定匹配顺序。
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

/// 检查一个确定的偏移和镜像窗口。
///
/// 参数完整描述匹配窗口，保留为独立值可以避免循环内分配临时结构。
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

/// 判断所有非空输入能否与无序原料建立一一对应关系。
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

/// 通过回溯为每个原料选择一个尚未使用且满足约束的输入。
///
/// 标签原料可能同时匹配多个物品，贪心选择会漏掉有效组合；输入数量受网格尺寸限制，
/// 因此这里用小规模回溯换取正确的一一匹配。
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
