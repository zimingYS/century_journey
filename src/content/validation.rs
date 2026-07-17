use crate::content::biome::BiomeDefinition;
use crate::content::block::definition::BlockProperty;
use crate::content::format::versioned_json_dir_results;
use crate::content::item::definition::ItemDefinition;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::loot::table::LootTable;
use crate::content::recipe::definition::Ingredient;
use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::tag::definition::TagAction;
use crate::engine::asset::{AssetFiles, AssetResolver};
use crate::shared::identifier::Identifier;
use std::collections::HashSet;

#[derive(Debug, Default)]
pub struct ContentCheckReport {
    pub checked_files: usize,
    pub errors: Vec<String>,
}

impl ContentCheckReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

pub fn check_content(resolver: &AssetResolver) -> ContentCheckReport {
    let files = AssetFiles::new(resolver);
    let mut report = ContentCheckReport::default();

    let blocks = load::<BlockProperty>(&files, "definitions/blocks", &mut report);
    let items = load::<ItemDefinition>(&files, "definitions/items", &mut report);
    let biomes = load::<BiomeDefinition>(&files, "definitions/biomes", &mut report);
    let recipes = load::<RecipeDefinition>(&files, "definitions/recipes", &mut report);
    let loot = load::<LootTable>(&files, "definitions/loot/blocks", &mut report);
    let tags = load::<TagAction>(&files, "definitions/tags", &mut report);

    let block_ids = unique_ids(
        blocks
            .iter()
            .map(|(path, block)| (path, block.identifier.to_string())),
        "block",
        &mut report,
    );
    let explicit_item_ids = unique_ids(
        items
            .iter()
            .map(|(path, item)| (path, item.identifier.to_string())),
        "item",
        &mut report,
    );
    let mut item_ids = explicit_item_ids.clone();
    item_ids.extend(block_ids.iter().cloned());

    if !block_ids.contains("century_journey:air") {
        report
            .errors
            .push("definitions/blocks: missing century_journey:air".into());
    }

    validate_blocks(resolver, &blocks, &block_ids, &mut report);
    validate_items(resolver, &items, &block_ids, &mut report);
    validate_biomes(&biomes, &block_ids, &mut report);
    validate_recipes(&recipes, &item_ids, &mut report);
    validate_loot(&loot, &block_ids, &item_ids, &mut report);
    validate_tags(&tags, &block_ids, &item_ids, &biomes, &mut report);

    report
}

fn load<T: serde::de::DeserializeOwned>(
    files: &AssetFiles<'_>,
    directory: &str,
    report: &mut ContentCheckReport,
) -> Vec<(String, T)> {
    let resolved = files.resolved_files(directory, "json");
    report.checked_files += resolved.len();
    versioned_json_dir_results::<T>(files, directory)
        .into_iter()
        .filter_map(|result| match result {
            Ok(value) => Some(value),
            Err(error) => {
                report.errors.push(error);
                None
            }
        })
        .collect()
}

fn unique_ids<'a>(
    entries: impl IntoIterator<Item = (&'a String, String)>,
    kind: &str,
    report: &mut ContentCheckReport,
) -> HashSet<String> {
    let mut ids = HashSet::new();
    for (path, identifier) in entries {
        if !ids.insert(identifier.clone()) {
            report
                .errors
                .push(format!("{path}: duplicate {kind} identifier {identifier}"));
        }
    }
    ids
}

fn validate_blocks(
    resolver: &AssetResolver,
    blocks: &[(String, BlockProperty)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, block) in blocks {
        if !block.hardness.is_finite() || block.hardness < 0.0 {
            report
                .errors
                .push(format!("{path}: hardness must be finite and non-negative"));
        }
        if block.light_emission > 15 {
            report
                .errors
                .push(format!("{path}: light_emission must be <= 15"));
        }
        for face in 0..6 {
            let texture = block.textures.get_face_texture(face);
            if !resolver.root_dir().join(texture).is_file() {
                report
                    .errors
                    .push(format!("{path}: missing block texture {texture}"));
            }
        }
        if let Some(drop) = &block.drop_identifier
            && !block_ids.contains(&drop.to_string())
        {
            report
                .errors
                .push(format!("{path}: unknown drop_identifier {drop}"));
        }
    }
}

fn validate_items(
    resolver: &AssetResolver,
    items: &[(String, ItemDefinition)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, item) in items {
        if item.max_stack == 0 {
            report
                .errors
                .push(format!("{path}: max_stack must be positive"));
        }
        if let Some(block) = &item.placeable_block
            && !block_ids.contains(&block.to_string())
        {
            report
                .errors
                .push(format!("{path}: unknown placeable_block {block}"));
        }
        match &item.icon {
            IconDefinition::Block(block) if !block_ids.contains(&block.to_string()) => {
                report
                    .errors
                    .push(format!("{path}: unknown block icon {block}"));
            }
            IconDefinition::Texture(identifier) => match Identifier::parse(identifier) {
                Ok(identifier) => {
                    let texture = resolver
                        .root_dir()
                        .join("textures/items")
                        .join(format!("{}.png", identifier.path()));
                    if !texture.is_file() {
                        report.errors.push(format!(
                            "{path}: missing item texture {}",
                            texture.display()
                        ));
                    }
                }
                Err(error) => report
                    .errors
                    .push(format!("{path}: invalid texture identifier: {error}")),
            },
            IconDefinition::Block(_) => {}
        }
    }
}

fn validate_biomes(
    biomes: &[(String, BiomeDefinition)],
    block_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    let mut ids = HashSet::new();
    let mut orders = HashSet::new();
    for (path, biome) in biomes {
        if !ids.insert(biome.identifier.to_string()) {
            report.errors.push(format!(
                "{path}: duplicate biome identifier {}",
                biome.identifier
            ));
        }
        if !orders.insert(biome.generation_order) {
            report.errors.push(format!(
                "{path}: duplicate biome generation_order {}",
                biome.generation_order
            ));
        }
        validate_unit_range(path, "temperature_range", biome.temperature_range, report);
        validate_unit_range(path, "humidity_range", biome.humidity_range, report);
        if !(0.0..=1.0).contains(&biome.tree_density) || !biome.tree_density.is_finite() {
            report
                .errors
                .push(format!("{path}: tree_density must be within 0..=1"));
        }
        for (field, block) in [
            ("surface_block", &biome.surface_block),
            ("subsurface_block", &biome.subsurface_block),
            ("beach_block", &biome.beach_block),
        ] {
            if !block_ids.contains(&block.to_string()) {
                report
                    .errors
                    .push(format!("{path}: unknown {field} {block}"));
            }
        }
    }
    if biomes.is_empty() {
        report
            .errors
            .push("definitions/biomes: at least one biome is required".into());
    }
}

fn validate_unit_range(
    path: &str,
    field: &str,
    range: (f64, f64),
    report: &mut ContentCheckReport,
) {
    if !range.0.is_finite()
        || !range.1.is_finite()
        || range.0 < 0.0
        || range.1 > 1.0
        || range.0 > range.1
    {
        report
            .errors
            .push(format!("{path}: {field} must be ordered within 0..=1"));
    }
}

fn validate_recipes(
    recipes: &[(String, RecipeDefinition)],
    item_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, recipe) in recipes {
        let (ingredients, result) = match recipe {
            RecipeDefinition::Shaped(recipe) => {
                if recipe.pattern.is_empty() {
                    report
                        .errors
                        .push(format!("{path}: shaped pattern is empty"));
                }
                let widths: HashSet<_> = recipe
                    .pattern
                    .iter()
                    .map(|row| row.chars().count())
                    .collect();
                if widths.len() > 1 || widths.contains(&0) {
                    report.errors.push(format!(
                        "{path}: shaped pattern rows must have one non-zero width"
                    ));
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
                            .push(format!("{path}: pattern key '{key}' has no ingredient"));
                    }
                }
                for key in recipe.key.keys() {
                    if *key == ' ' || !used_keys.contains(key) {
                        report
                            .errors
                            .push(format!("{path}: unused or reserved recipe key '{key}'"));
                    }
                }
                (recipe.key.values().collect::<Vec<_>>(), &recipe.result)
            }
            RecipeDefinition::Shapeless(recipe) => {
                if recipe.ingredients.is_empty() {
                    report
                        .errors
                        .push(format!("{path}: shapeless ingredients are empty"));
                }
                (
                    recipe.ingredients.iter().collect::<Vec<_>>(),
                    &recipe.result,
                )
            }
        };
        for ingredient in ingredients {
            if let Ingredient::Item { item } = ingredient
                && !item_ids.contains(&item.to_string())
            {
                report
                    .errors
                    .push(format!("{path}: unknown ingredient item {item}"));
            }
        }
        if result.count == 0 {
            report
                .errors
                .push(format!("{path}: recipe result count is zero"));
        }
        if !item_ids.contains(&result.item.to_string()) {
            report
                .errors
                .push(format!("{path}: unknown recipe result {}", result.item));
        }
    }
}

fn validate_loot(
    tables: &[(String, LootTable)],
    block_ids: &HashSet<String>,
    item_ids: &HashSet<String>,
    report: &mut ContentCheckReport,
) {
    for (path, table) in tables {
        let block_id = definition_id(path, "definitions/loot/blocks/");
        if !block_ids.contains(&block_id) {
            report.errors.push(format!(
                "{path}: loot table targets unknown block {block_id}"
            ));
        }
        for entry in &table.entries {
            if !item_ids.contains(&entry.item.to_string()) {
                report
                    .errors
                    .push(format!("{path}: unknown loot item {}", entry.item));
            }
            if entry.min_count > entry.max_count {
                report
                    .errors
                    .push(format!("{path}: loot count range is reversed"));
            }
            if !(0.0..=1.0).contains(&entry.chance) || !entry.chance.is_finite() {
                report
                    .errors
                    .push(format!("{path}: loot chance must be within 0..=1"));
            }
        }
    }
}

fn validate_tags(
    tags: &[(String, TagAction)],
    block_ids: &HashSet<String>,
    item_ids: &HashSet<String>,
    biomes: &[(String, BiomeDefinition)],
    report: &mut ContentCheckReport,
) {
    let biome_ids: HashSet<_> = biomes
        .iter()
        .map(|(_, biome)| biome.identifier.to_string())
        .collect();
    let tag_keys: HashSet<_> = tags
        .iter()
        .filter_map(|(path, _)| tag_identity(path))
        .collect();
    for (path, action) in tags {
        let Some((kind, _)) = tag_identity(path).and_then(|value| {
            value
                .split_once(':')
                .map(|(kind, rest)| (kind.to_string(), rest.to_string()))
        }) else {
            report.errors.push(format!("{path}: invalid tag path"));
            continue;
        };
        let known = match kind.as_str() {
            "block" => block_ids,
            "item" => item_ids,
            "biome" => &biome_ids,
            _ => {
                report
                    .errors
                    .push(format!("{path}: unsupported tag kind {kind}"));
                continue;
            }
        };
        for value in tag_values(action) {
            if let Some(reference) = value.strip_prefix('#') {
                if !tag_keys.contains(&format!("{kind}:{reference}")) {
                    report
                        .errors
                        .push(format!("{path}: unresolved tag reference {value}"));
                }
            } else if !known.contains(value) {
                report
                    .errors
                    .push(format!("{path}: unknown {kind} tag member {value}"));
            }
        }
    }
}

fn tag_values(action: &TagAction) -> &[String] {
    match action {
        TagAction::Append { append } => append,
        TagAction::Remove { remove } => remove,
        TagAction::Replace { replace } => replace,
        TagAction::Values { values, .. } => values,
    }
}

fn tag_identity(path: &str) -> Option<String> {
    let relative = path.strip_prefix("definitions/tags/")?;
    let mut parts = relative.split('/');
    let kind = parts.next()?;
    let namespace = parts.next()?;
    let name = parts.collect::<Vec<_>>().join("/");
    (!name.is_empty()).then(|| format!("{kind}:{namespace}:{name}"))
}

fn definition_id(path: &str, prefix: &str) -> String {
    let relative = path.strip_prefix(prefix).unwrap_or(path);
    relative
        .split_once('/')
        .map(|(namespace, name)| format!("{namespace}:{name}"))
        .unwrap_or_else(|| format!("century_journey:{relative}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repository_content_is_valid() {
        let resolver =
            AssetResolver::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
        let report = check_content(&resolver);
        assert!(report.errors.is_empty(), "{}", report.errors.join("\n"));
    }

    #[test]
    fn later_content_source_overrides_the_same_relative_path() {
        let root = std::env::temp_dir().join(format!(
            "century_journey_content_override_{}",
            std::process::id()
        ));
        let base = root.join("base");
        let override_root = root.join("override");
        let relative = std::path::Path::new("definitions/items/example.json");
        std::fs::create_dir_all(base.join("definitions/items")).unwrap();
        std::fs::create_dir_all(override_root.join("definitions/items")).unwrap();
        std::fs::write(base.join(relative), "{}").unwrap();
        std::fs::write(override_root.join(relative), r#"{"override":true}"#).unwrap();

        let resolver = AssetResolver::with_content_overrides(&base, [override_root.clone()]);
        let files = AssetFiles::new(&resolver).resolved_files("definitions/items", "json");
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].full_path, override_root.join(relative));

        std::fs::remove_dir_all(root).unwrap();
    }
}
