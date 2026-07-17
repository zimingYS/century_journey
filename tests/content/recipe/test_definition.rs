use century_journey::content::format::Versioned;
use century_journey::content::recipe::definition::recipe_definition::RecipeDefinition;
use serde_json::json;

#[test]
fn deserialize_shaped_recipe() {
    let value = json!({
        "format_version": 1,
        "type": "shaped",
        "pattern": [
            "AA"
        ],
        "key": {
            "A": {
                "item": "century_journey:wood"
            }
        },
        "result": {
            "item": "century_journey:wood",
            "count": 4
        }
    });

    let recipe: Versioned<RecipeDefinition> = serde_json::from_value(value).unwrap();

    match recipe.content {
        RecipeDefinition::Shaped(recipe) => {
            assert_eq!(recipe.pattern.len(), 1);
            assert_eq!(recipe.result.count, 4);
        }
        _ => panic!("Expected shaped recipe"),
    }
}
