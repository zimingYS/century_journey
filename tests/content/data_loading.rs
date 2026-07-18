use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use century_journey::content::block::registry::{BlockRegistry, init_block_registry_system};
use century_journey::content::item::definition::{ItemCategory, ItemDefinition};
use century_journey::content::item::registry::registry::ItemRegistry;
use century_journey::content::loot::loader::load_loot_tables;
use century_journey::content::loot::table::LootTable;
use century_journey::content::recipe::loader::load_recipe_definitions;
use century_journey::content::recipe::registry::RecipeRegistry;
use century_journey::content::tag::compiler::TagRegistryCompiler;
use century_journey::content::tag::loader::load_tag_actions;
use century_journey::content::validation::{
    ContentCheckReport, ContentCompilation, compile_content,
};
use century_journey::engine::asset::{AssetFiles, AssetManager};
use century_journey::shared::identifier::Identifier;
use century_journey::shared::item_id::ItemId;
use century_journey::shared::states::AppState;
use century_journey::shared::tag::identifier::TagId;

fn load_block_registry() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<AppState>();
    app.init_resource::<AssetManager>();
    let compilation = {
        let asset = app.world().resource::<AssetManager>();
        compile_content(asset.resolver())
    };
    assert!(
        compilation.is_valid(),
        "{}",
        compilation.error_summary(usize::MAX)
    );
    app.insert_resource(compilation);
    app.add_systems(Update, init_block_registry_system);
    app.update();
    app
}

#[test]
fn block_json_scan_registers_runtime_block_registry() {
    let app = load_block_registry();
    let registry = app.world().resource::<BlockRegistry>();

    assert_eq!(
        registry.get_id_by_identifier("century_journey:air"),
        Some(0)
    );
    assert!(
        registry
            .get_id_by_identifier("century_journey:stone")
            .is_some()
    );
    assert!(registry.iter_properties().count() >= 10);
    assert!(registry.total_layer_count() > 0);
}

#[test]
fn invalid_compilation_never_builds_runtime_block_registry() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<AppState>();
    app.insert_resource(ContentCompilation {
        report: ContentCheckReport {
            checked_files: 1,
            errors: vec!["definitions/blocks/bad:hardness: invalid".into()],
        },
        ..default()
    });
    app.add_systems(Update, init_block_registry_system);

    app.update();

    assert!(!app.world().contains_resource::<BlockRegistry>());
}

#[test]
fn item_json_scan_registers_item_registry() {
    let asset = AssetManager::default();
    let files = AssetFiles::new(asset.resolver());
    let definitions = files.read_json_dir::<ItemDefinition>("definitions/items");

    assert!(!definitions.is_empty());

    let mut registry = ItemRegistry::default();
    for (_, definition) in definitions {
        registry.register(definition);
    }

    let stick = ItemId::item("century_journey:stick");
    assert!(registry.contains(&stick));
    assert!(!registry.items_by_category(&ItemCategory::Tool).is_empty());
}

#[test]
fn loot_json_scan_loads_block_loot_tables() {
    let asset = AssetManager::default();
    let tables = load_loot_tables(&asset);
    let grass = Identifier::parse("century_journey:grass").unwrap();

    assert!(!tables.is_empty());
    assert!(
        tables
            .get(&grass)
            .is_some_and(|table| !table.entries.is_empty())
    );

    let files = AssetFiles::new(asset.resolver());
    let raw_tables = files.read_json_dir::<LootTable>("definitions/loot/blocks");
    assert_eq!(raw_tables.len(), tables.len());
}

#[test]
fn tag_json_scan_compiles_runtime_registry() {
    let app = load_block_registry();
    let block_registry = app.world().resource::<BlockRegistry>();
    let asset = AssetManager::default();
    let actions = load_tag_actions(&asset);

    assert!(!actions.is_empty());

    let mut compiler = TagRegistryCompiler::new();
    compiler.collect_from_blocks(block_registry);
    for (tag_id, action) in actions {
        compiler.apply_action(tag_id, &action);
    }
    compiler.resolve_references();

    let (runtime, item_index) = compiler.build_runtime(block_registry);
    let tree_plantable = TagId::new("century_journey", "tree_plantable");
    let grass_id = block_registry
        .get_id_by_identifier("century_journey:grass")
        .unwrap();

    assert!(runtime.total_tags() > 0);
    assert!(runtime.contains(&tree_plantable, grass_id));
    assert!(item_index.total_tags() > 0);
}

#[test]
fn recipe_json_scan_registers_recipe_registry() {
    let asset = AssetManager::default();
    let recipes = load_recipe_definitions(&asset);

    assert!(!recipes.is_empty());

    let mut registry = RecipeRegistry::default();
    for (id, recipe) in recipes {
        registry.register(id, recipe);
    }

    let recipe_id = Identifier::parse("century_journey:stick_from_planks").unwrap();
    assert!(registry.get(&recipe_id).is_some());
    assert!(registry.all_recipes().count() >= 1);
}
