//! 配方形状、材料引用和产出定义校验。

use super::super::ContentCheckReport;
use crate::content::recipe::definition::Ingredient;
use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use std::collections::HashSet;

/// 校验配方形状、材料引用与产出数量。
pub(in crate::content::validation) fn validate_recipes(
    recipes: &[(String, RecipeDefinition)],
    item_ids: &HashSet<String>,
    item_tag_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, recipe) in recipes {
        let (ingredients, result) = match recipe {
            RecipeDefinition::Shaped(recipe) => {
                if recipe.pattern.is_empty() {
                    report
                        .errors
                        .push(format!("{path}:pattern: must not be empty"));
                }
                let widths: HashSet<_> = recipe
                    .pattern
                    .iter()
                    .map(|row| row.chars().count())
                    .collect();
                if widths.len() > 1 || widths.contains(&0) {
                    report
                        .errors
                        .push(format!("{path}:pattern: rows must have one non-zero width"));
                }
                let used_keys: HashSet<char> = recipe
                    .pattern
                    .iter()
                    .flat_map(|row| row.chars())
                    .filter(|key| *key != ' ')
                    .collect();
                for key in &used_keys {
                    if !recipe.key.contains_key(key) {
                        report
                            .errors
                            .push(format!("{path}:key.{key}: missing ingredient"));
                    }
                }
                for key in recipe.key.keys() {
                    if *key == ' ' || !used_keys.contains(key) {
                        report
                            .errors
                            .push(format!("{path}:key.{key}: unused or reserved key"));
                    }
                }
                (recipe.key.values().collect::<Vec<_>>(), &recipe.result)
            }
            RecipeDefinition::Shapeless(recipe) => {
                if recipe.ingredients.is_empty() {
                    report
                        .errors
                        .push(format!("{path}:ingredients: must not be empty"));
                }
                (
                    recipe.ingredients.iter().collect::<Vec<_>>(),
                    &recipe.result,
                )
            }
        };
        for ingredient in ingredients {
            match ingredient {
                Ingredient::Item { item } if !item_ids.contains(&item.to_string()) => {
                    report
                        .errors
                        .push(format!("{path}:ingredients: unknown item {item}"));
                }
                Ingredient::Tag { tag } if !item_tag_ids.contains(&tag.to_full()) => {
                    report.errors.push(format!(
                        "{path}:ingredients: unknown item tag {}",
                        tag.to_full()
                    ));
                }
                _ => {}
            }
        }
        if result.count == 0 {
            report
                .errors
                .push(format!("{path}:result.count: must be positive"));
        }
        if !item_ids.contains(&result.item.to_string()) {
            report
                .errors
                .push(format!("{path}:result.item: unknown item {}", result.item));
        }
    }
}
