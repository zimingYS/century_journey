use crate::content::biome::BiomeDefinition;
use crate::content::block::definition::BlockProperty;
use crate::content::block::model::BlockModel;
use crate::content::format::versioned_json_dir_results;
use crate::content::item::definition::ItemDefinition;
use crate::content::item::texture::icon::IconDefinition;
use crate::content::loot::table::LootTable;
use crate::content::recipe::definition::Ingredient;
use crate::content::recipe::definition::recipe_definition::RecipeDefinition;
use crate::content::tag::definition::TagAction;
use crate::engine::asset::{AssetFiles, AssetResolver};
use crate::shared::held_item::HeldRenderDefinition;
use crate::shared::identifier::Identifier;
use crate::shared::tag::identifier::TagId;
use bevy::prelude::Resource;
use std::collections::HashSet;

#[derive(Debug, Clone, Default)]
pub struct ContentCheckReport {
    pub checked_files: usize,
    pub errors: Vec<String>,
}

impl ContentCheckReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// 全局校验完成后唯一允许进入运行时注册表的数据集合。
///
/// 每个列表都已按稳定标识符排序，运行时动态 ID 不再取决于文件系统遍历顺序。
#[derive(Debug, Clone, Default)]
pub struct CompiledContent {
    pub blocks: Vec<BlockProperty>,
    pub items: Vec<ItemDefinition>,
    pub biomes: Vec<BiomeDefinition>,
    pub recipes: Vec<(Identifier, RecipeDefinition)>,
    pub block_loot: Vec<(Identifier, LootTable)>,
    pub tags: Vec<(TagId, TagAction)>,
}

#[derive(Resource, Debug, Clone, Default)]
pub struct ContentCompilation {
    pub report: ContentCheckReport,
    pub content: CompiledContent,
}

impl ContentCompilation {
    pub fn is_valid(&self) -> bool {
        self.report.is_valid()
    }

    pub fn error_summary(&self, limit: usize) -> String {
        let shown = self
            .report
            .errors
            .iter()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        let remaining = self.report.errors.len().saturating_sub(limit);
        if remaining == 0 {
            shown
        } else {
            format!("{shown}\n... 另有 {remaining} 个错误")
        }
    }
}

pub fn check_content(resolver: &AssetResolver) -> ContentCheckReport {
    compile_content(resolver).report
}

pub fn compile_content(resolver: &AssetResolver) -> ContentCompilation {
    let files = AssetFiles::new(resolver);
    let mut report = ContentCheckReport::default();

    let mut blocks = load::<BlockProperty>(&files, "definitions/blocks", &mut report);
    let mut items = load::<ItemDefinition>(&files, "definitions/items", &mut report);
    let mut biomes = load::<BiomeDefinition>(&files, "definitions/biomes", &mut report);
    let mut recipes = load::<RecipeDefinition>(&files, "definitions/recipes", &mut report);
    let mut loot = load::<LootTable>(&files, "definitions/loot/blocks", &mut report);
    let mut tags = load::<TagAction>(&files, "definitions/tags", &mut report);

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
    for block_id in &block_ids {
        if explicit_item_ids.contains(block_id) {
            report.errors.push(format!(
                "definitions/items:identifier: explicit item {block_id} conflicts with the generated block item"
            ));
        }
        item_ids.insert(block_id.clone());
    }

    if !block_ids.contains("century_journey:air") {
        report
            .errors
            .push("definitions/blocks: missing century_journey:air".into());
    }

    validate_blocks(resolver, &blocks, &block_ids, &mut report);
    validate_items(resolver, &items, &block_ids, &mut report);
    validate_biomes(&biomes, &block_ids, &mut report);
    let mut item_tag_ids = tags
        .iter()
        .filter_map(|(path, _)| tag_identity(path))
        .filter_map(|identity| identity.strip_prefix("item:").map(ToOwned::to_owned))
        .collect::<HashSet<_>>();
    for (_, item) in &items {
        item_tag_ids.extend(item.tags.iter().map(|tag| inline_tag_id(tag).to_full()));
    }
    validate_recipes(&recipes, &item_ids, &item_tag_ids, &mut report);
    validate_loot(&loot, &block_ids, &item_ids, &mut report);
    validate_tags(&tags, &block_ids, &item_ids, &biomes, &mut report);
    validate_textures(resolver, &files, &mut report);

    blocks.sort_by(|left, right| {
        left.1
            .identifier
            .cmp(&right.1.identifier)
            .then_with(|| left.0.cmp(&right.0))
    });
    items.sort_by(|left, right| {
        left.1
            .identifier
            .cmp(&right.1.identifier)
            .then_with(|| left.0.cmp(&right.0))
    });
    biomes.sort_by(|left, right| {
        left.1
            .generation_order
            .cmp(&right.1.generation_order)
            .then_with(|| left.1.identifier.cmp(&right.1.identifier))
            .then_with(|| left.0.cmp(&right.0))
    });
    recipes.sort_by(|left, right| left.0.cmp(&right.0));
    loot.sort_by(|left, right| left.0.cmp(&right.0));
    tags.sort_by(|left, right| left.0.cmp(&right.0));

    let mut recipe_ids = HashSet::new();
    let recipes = recipes
        .into_iter()
        .filter_map(|(path, recipe)| {
            let Some(identifier) = recipe_id(&path) else {
                report
                    .errors
                    .push(format!("{path}:path: invalid recipe definition path"));
                return None;
            };
            if !recipe_ids.insert(identifier.clone()) {
                report.errors.push(format!(
                    "{path}:path: duplicate recipe identifier {identifier}"
                ));
                return None;
            }
            Some((identifier, recipe))
        })
        .collect();
    let mut loot_ids = HashSet::new();
    let block_loot = loot
        .into_iter()
        .filter_map(|(path, table)| {
            let Some(identifier) = block_loot_id(&path) else {
                report
                    .errors
                    .push(format!("{path}:path: invalid block loot definition path"));
                return None;
            };
            if !loot_ids.insert(identifier.clone()) {
                report.errors.push(format!(
                    "{path}:path: duplicate block loot identifier {identifier}"
                ));
                return None;
            }
            Some((identifier, table))
        })
        .collect();
    let tags = tags
        .into_iter()
        .filter_map(|(path, action)| {
            tag_runtime_id(&path).map(|id| (id, action)).or_else(|| {
                report
                    .errors
                    .push(format!("{path}:path: invalid tag definition path"));
                None
            })
        })
        .collect();

    ContentCompilation {
        report,
        content: CompiledContent {
            blocks: blocks.into_iter().map(|(_, value)| value).collect(),
            items: items.into_iter().map(|(_, value)| value).collect(),
            biomes: biomes.into_iter().map(|(_, value)| value).collect(),
            recipes,
            block_loot,
            tags,
        },
    }
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
            report.errors.push(format!(
                "{path}:identifier: duplicate {kind} identifier {identifier}"
            ));
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
                .push(format!("{path}:hardness: must be finite and non-negative"));
        }
        if block.light_emission > 15 {
            report
                .errors
                .push(format!("{path}:light_emission: must be <= 15"));
        }
        if !block.light_transmission.is_finite() || !(0.0..=1.0).contains(&block.light_transmission)
        {
            report.errors.push(format!(
                "{path}:light_transmission: must be finite and within 0..=1"
            ));
        }
        for face in 0..6 {
            let texture = block.textures.get_face_texture(face);
            if !content_file_exists(resolver, texture) {
                report.errors.push(format!(
                    "{path}:textures.{face}: missing block texture {texture}"
                ));
            }
        }
        if let Some(drop) = &block.drop_identifier
            && !block_ids.contains(&drop.to_string())
        {
            report
                .errors
                .push(format!("{path}:drop_identifier: unknown block {drop}"));
        }
        for (field, volume) in [
            ("sound.break_volume", block.sound.break_volume),
            ("sound.place_volume", block.sound.place_volume),
            ("sound.step_volume", block.sound.step_volume),
        ] {
            if !volume.is_finite() || !(0.0..=1.0).contains(&volume) {
                report
                    .errors
                    .push(format!("{path}:{field}: must be finite and within 0..=1"));
            }
        }
        validate_block_model(path, &block.model.model, report);
        let mut property_names = HashSet::new();
        let mut state_count = 1usize;
        for (index, property) in block.states.properties.iter().enumerate() {
            let field = format!("states.properties[{index}]");
            if property.name.is_empty() || !property_names.insert(property.name.as_str()) {
                report
                    .errors
                    .push(format!("{path}:{field}.name: must be non-empty and unique"));
            }
            if property.values.is_empty() {
                report
                    .errors
                    .push(format!("{path}:{field}.values: must not be empty"));
                continue;
            }
            if property.default_index >= property.values.len() {
                report.errors.push(format!(
                    "{path}:{field}.default_index: exceeds values length {}",
                    property.values.len()
                ));
            }
            let unique_values = property.values.iter().collect::<HashSet<_>>();
            if unique_values.len() != property.values.len() {
                report
                    .errors
                    .push(format!("{path}:{field}.values: contains duplicates"));
            }
            state_count = state_count.saturating_mul(property.values.len());
        }
        if state_count > u16::MAX as usize + 1 {
            report.errors.push(format!(
                "{path}:states.properties: {state_count} combinations exceed the u16 state space"
            ));
        }
    }
}

fn validate_block_model(path: &str, model: &BlockModel, report: &mut ContentCheckReport) {
    match model {
        BlockModel::Slab { thickness }
            if !thickness.is_finite() || !(0.0..=1.0).contains(thickness) =>
        {
            report.errors.push(format!(
                "{path}:model.model.thickness: must be finite and within 0..=1"
            ));
        }
        BlockModel::Custom { faces } => {
            for (index, face) in faces.iter().enumerate() {
                if face
                    .vertices
                    .iter()
                    .flatten()
                    .any(|coordinate| !coordinate.is_finite())
                    || face.normal.iter().any(|coordinate| !coordinate.is_finite())
                {
                    report.errors.push(format!(
                        "{path}:model.model.faces[{index}]: vertices and normal must be finite"
                    ));
                }
                if !face.ambient_occlusion.is_finite()
                    || !(0.0..=1.0).contains(&face.ambient_occlusion)
                {
                    report.errors.push(format!(
                        "{path}:model.model.faces[{index}].ambient_occlusion: must be finite and within 0..=1"
                    ));
                }
            }
        }
        _ => {}
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
                .push(format!("{path}:max_stack: must be positive"));
        }
        if let Some(block) = &item.placeable_block
            && !block_ids.contains(&block.to_string())
        {
            report
                .errors
                .push(format!("{path}:placeable_block: unknown block {block}"));
        }
        if let Some(tool) = &item.tool {
            if tool.max_durability == 0 {
                report
                    .errors
                    .push(format!("{path}:tool.max_durability: must be positive"));
            }
            if !tool.efficiency.is_finite() || tool.efficiency <= 0.0 {
                report.errors.push(format!(
                    "{path}:tool.efficiency: must be finite and positive"
                ));
            }
        }
        if let Some(food) = &item.food
            && (!food.hunger.is_finite()
                || !food.saturation.is_finite()
                || food.hunger < 0.0
                || food.saturation < 0.0)
        {
            report.errors.push(format!(
                "{path}:food: hunger and saturation must be finite and non-negative"
            ));
        }
        match &item.held_renderer {
            HeldRenderDefinition::FlatItem { thickness }
                if !thickness.is_finite() || *thickness <= 0.0 =>
            {
                report.errors.push(format!(
                    "{path}:held_renderer.thickness: must be finite and positive"
                ));
            }
            HeldRenderDefinition::Model { path: model_path } if model_path.trim().is_empty() => {
                report
                    .errors
                    .push(format!("{path}:held_renderer.path: must not be empty"));
            }
            _ => {}
        }
        match &item.icon {
            IconDefinition::Block(block) if !block_ids.contains(&block.to_string()) => {
                report
                    .errors
                    .push(format!("{path}:icon.value: unknown block {block}"));
            }
            IconDefinition::Texture(identifier) => match Identifier::parse(identifier) {
                Ok(identifier) => {
                    let texture = format!("textures/items/{}.png", identifier.path());
                    if !content_file_exists(resolver, &texture) {
                        report
                            .errors
                            .push(format!("{path}:icon.value: missing item texture {texture}"));
                    }
                }
                Err(error) => report.errors.push(format!(
                    "{path}:icon.value: invalid texture identifier: {error}"
                )),
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
                "{path}:identifier: duplicate biome identifier {}",
                biome.identifier
            ));
        }
        if !orders.insert(biome.generation_order) {
            report.errors.push(format!(
                "{path}:generation_order: duplicate value {}",
                biome.generation_order
            ));
        }
        validate_unit_range(path, "temperature_range", biome.temperature_range, report);
        validate_unit_range(path, "humidity_range", biome.humidity_range, report);
        if !(0.0..=1.0).contains(&biome.tree_density) || !biome.tree_density.is_finite() {
            report
                .errors
                .push(format!("{path}:tree_density: must be within 0..=1"));
        }
        if !biome.terrain.base_height.is_finite() {
            report
                .errors
                .push(format!("{path}:terrain.base_height: must be finite"));
        }
        if !biome.terrain.height_amplitude.is_finite() || biome.terrain.height_amplitude < 0.0 {
            report.errors.push(format!(
                "{path}:terrain.height_amplitude: must be finite and non-negative"
            ));
        }
        if !biome.terrain.roughness.is_finite() || biome.terrain.roughness < 0.0 {
            report.errors.push(format!(
                "{path}:terrain.roughness: must be finite and non-negative"
            ));
        }
        for (field, block) in [
            ("surface_block", &biome.surface_block),
            ("subsurface_block", &biome.subsurface_block),
            ("beach_block", &biome.beach_block),
        ] {
            if !block_ids.contains(&block.to_string()) {
                report
                    .errors
                    .push(format!("{path}:{field}: unknown block {block}"));
            }
        }
    }
    if biomes.is_empty() {
        report
            .errors
            .push("definitions/biomes:directory: at least one biome is required".into());
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
            .push(format!("{path}:{field}: must be ordered within 0..=1"));
    }
}

fn validate_recipes(
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
                "{path}:path: loot table targets unknown block {block_id}"
            ));
        }
        for (index, entry) in table.entries.iter().enumerate() {
            if !item_ids.contains(&entry.item.to_string()) {
                report.errors.push(format!(
                    "{path}:entries[{index}].item: unknown item {}",
                    entry.item
                ));
            }
            if entry.min_count > entry.max_count {
                report.errors.push(format!(
                    "{path}:entries[{index}].min_count: exceeds max_count"
                ));
            }
            if entry.max_count == 0 {
                report.errors.push(format!(
                    "{path}:entries[{index}].max_count: must be positive"
                ));
            }
            if !(0.0..=1.0).contains(&entry.chance) || !entry.chance.is_finite() {
                report.errors.push(format!(
                    "{path}:entries[{index}].chance: must be finite and within 0..=1"
                ));
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
            report.errors.push(format!("{path}:path: invalid tag path"));
            continue;
        };
        let known = match kind.as_str() {
            "block" => block_ids,
            "item" => item_ids,
            "biome" => &biome_ids,
            _ => {
                report
                    .errors
                    .push(format!("{path}:path: unsupported tag kind {kind}"));
                continue;
            }
        };
        for value in tag_values(action) {
            if let Some(reference) = value.strip_prefix('#') {
                if !tag_keys.contains(&format!("{kind}:{reference}")) {
                    report
                        .errors
                        .push(format!("{path}:values: unresolved tag reference {value}"));
                }
            } else if !known.contains(value) {
                report
                    .errors
                    .push(format!("{path}:values: unknown {kind} member {value}"));
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

fn content_file_exists(resolver: &AssetResolver, relative: &str) -> bool {
    resolver
        .content_roots()
        .iter()
        .rev()
        .any(|root| root.join(relative).is_file())
}

fn validate_textures(
    _resolver: &AssetResolver,
    files: &AssetFiles<'_>,
    report: &mut ContentCheckReport,
) {
    for directory in ["textures/blocks", "textures/items"] {
        let textures = files.resolved_files(directory, "png");
        report.checked_files += textures.len();
        for texture in textures {
            match std::fs::read(&texture.full_path) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(image) if image.width() > 0 && image.height() > 0 => {}
                    Ok(_) => report.errors.push(format!(
                        "{}:image.dimensions: width and height must be positive",
                        texture.full_path.display()
                    )),
                    Err(error) => report.errors.push(format!(
                        "{}:image.data: cannot decode PNG: {error}",
                        texture.full_path.display()
                    )),
                },
                Err(error) => report.errors.push(format!(
                    "{}:image.data: cannot read texture: {error}",
                    texture.full_path.display()
                )),
            }
        }
    }
}

fn recipe_id(path: &str) -> Option<Identifier> {
    definition_identifier(path, "definitions/recipes/", None)
}

fn block_loot_id(path: &str) -> Option<Identifier> {
    definition_identifier(path, "definitions/loot/blocks/", Some("century_journey"))
}

fn definition_identifier(
    path: &str,
    prefix: &str,
    default_namespace: Option<&str>,
) -> Option<Identifier> {
    let relative = path.strip_prefix(prefix)?.replace('\\', "/");
    if let Some((namespace, name)) = relative.split_once('/') {
        (!namespace.is_empty() && !name.is_empty()).then(|| Identifier::new(namespace, name))
    } else {
        default_namespace.map(|namespace| Identifier::new(namespace, relative))
    }
}

fn tag_runtime_id(path: &str) -> Option<TagId> {
    let relative = path.strip_prefix("definitions/tags/")?.replace('\\', "/");
    let mut parts = relative.split('/');
    let _kind = parts.next()?;
    let namespace = parts.next()?;
    let name = parts.collect::<Vec<_>>().join("/");
    (!namespace.is_empty() && !name.is_empty()).then(|| TagId::new(namespace, name))
}

fn inline_tag_id(tag: &str) -> TagId {
    if let Some((namespace, path)) = tag.split_once('/') {
        TagId::new(namespace, path)
    } else {
        TagId::new("century_journey", tag)
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
    fn compiled_registries_are_sorted_by_stable_identity() {
        let resolver =
            AssetResolver::new(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
        let compilation = compile_content(&resolver);
        assert!(
            compilation.is_valid(),
            "{}",
            compilation.error_summary(usize::MAX)
        );

        assert!(
            compilation
                .content
                .blocks
                .windows(2)
                .all(|pair| { pair[0].identifier <= pair[1].identifier })
        );
        assert!(
            compilation
                .content
                .items
                .windows(2)
                .all(|pair| { pair[0].identifier <= pair[1].identifier })
        );
        assert!(
            compilation
                .content
                .recipes
                .windows(2)
                .all(|pair| { pair[0].0 <= pair[1].0 })
        );
        assert!(
            compilation
                .content
                .block_loot
                .windows(2)
                .all(|pair| { pair[0].0 <= pair[1].0 })
        );
    }

    #[test]
    fn dangling_reference_reports_file_and_field_path() {
        let root = std::env::temp_dir().join(format!(
            "century_journey_content_dangling_{}",
            std::process::id()
        ));
        let override_file = root.join("definitions/loot/blocks/stone.json");
        std::fs::create_dir_all(override_file.parent().unwrap()).unwrap();
        std::fs::write(
            &override_file,
            r#"{
                "format_version": 1,
                "entries": [{
                    "item": "century_journey:oak_sapling",
                    "min_count": 1,
                    "max_count": 1,
                    "chance": 1.0
                }]
            }"#,
        )
        .unwrap();
        let resolver = AssetResolver::with_content_overrides(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            [root.clone()],
        );

        let compilation = compile_content(&resolver);

        assert!(!compilation.is_valid());
        assert!(compilation.report.errors.iter().any(|error| {
            error.contains("definitions/loot/blocks/stone:entries[0].item")
                && error.contains("oak_sapling")
        }));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn invalid_png_is_part_of_global_content_validation() {
        let root = std::env::temp_dir().join(format!(
            "century_journey_content_texture_{}",
            std::process::id()
        ));
        let override_file = root.join("textures/items/broken.png");
        std::fs::create_dir_all(override_file.parent().unwrap()).unwrap();
        std::fs::write(&override_file, b"not a png").unwrap();
        let resolver = AssetResolver::with_content_overrides(
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
            [root.clone()],
        );

        let compilation = compile_content(&resolver);

        assert!(!compilation.is_valid());
        assert!(
            compilation
                .report
                .errors
                .iter()
                .any(|error| { error.contains("broken.png:image.data: cannot decode PNG") })
        );
        std::fs::remove_dir_all(root).unwrap();
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
