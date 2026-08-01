use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use century_journey::content::block::registry::{BlockRegistry, init_block_registry_system};
use century_journey::content::validation::{ContentCompilation, compile_content};
use century_journey::content::vegetation::registry::TreeSpeciesRegistry;
use century_journey::engine::asset::{AssetManager, AssetResolver};
use century_journey::shared::states::AppState;

fn load_content_and_blocks() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin));
    app.init_state::<AppState>();
    app.init_resource::<AssetManager>();
    let compilation = {
        let assets = app.world().resource::<AssetManager>();
        compile_content(assets.resolver())
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
fn repository_tree_species_builds_a_runtime_registry() {
    let app = load_content_and_blocks();
    let definitions = app
        .world()
        .resource::<ContentCompilation>()
        .content
        .tree_species
        .clone();
    let block_registry = app.world().resource::<BlockRegistry>();
    let sapling_id = block_registry
        .get_id_by_identifier("century_journey:sapling")
        .unwrap();
    let mut registry = TreeSpeciesRegistry::default();

    registry
        .replace_definitions(definitions, block_registry)
        .unwrap();

    let oak = registry.get_by_sapling_id(sapling_id).unwrap();
    assert_eq!(oak.definition.identifier.to_string(), "century_journey:oak");
    assert_eq!(oak.definition.growth.sapling_duration_game_minutes, 24 * 60);
    assert_eq!(
        oak.definition.growth.young_duration_game_minutes,
        3 * 24 * 60
    );
    assert_eq!(oak.definition.growth.retry_interval_game_minutes, 5);
    assert!(oak.definition.young_blueprint.is_some());
    assert_eq!(
        block_registry
            .get_identifier_by_id(oak.trunk_block_id)
            .unwrap()
            .to_string(),
        "century_journey:wood"
    );
}

#[test]
fn invalid_tree_species_reports_reference_and_growth_fields() {
    let root = std::env::temp_dir().join(format!(
        "century_journey_content_tree_species_{}",
        std::process::id()
    ));
    let override_file = root.join("definitions/tree_species/century_journey/oak.json");
    std::fs::create_dir_all(override_file.parent().unwrap()).unwrap();
    std::fs::write(
        &override_file,
        r#"{
            "format_version": 1,
            "identifier": "century_journey:oak",
            "display_name": "Broken Oak",
            "sapling_block": "century_journey:sapling",
            "trunk_block": "century_journey:missing_wood",
            "leaves_block": "century_journey:leaves",
            "growth": {
                "sapling_duration_game_minutes": 0,
                "young_duration_game_minutes": 0,
                "retry_interval_game_minutes": 0
            },
            "young_blueprint": {
                "trunk_height": { "min": 0, "max": 2 },
                "crown_radius": { "min": 4, "max": 2 }
            },
            "blueprint": {
                "trunk_height": { "min": 0, "max": 80 },
                "crown_radius": { "min": 4, "max": 2 }
            }
        }"#,
    )
    .unwrap();
    let resolver = AssetResolver::with_content_overrides(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        [root.clone()],
    );

    let compilation = compile_content(&resolver);

    assert!(!compilation.is_valid());
    for expected_field in [
        "trunk_block",
        "growth.sapling_duration_game_minutes",
        "growth.young_duration_game_minutes",
        "growth.retry_interval_game_minutes",
        "young_blueprint.trunk_height",
        "young_blueprint.crown_radius",
        "blueprint.trunk_height",
        "blueprint.crown_radius",
    ] {
        assert!(
            compilation
                .report
                .errors
                .iter()
                .any(|error| error.contains(expected_field)),
            "missing diagnostic for {expected_field}"
        );
    }
    std::fs::remove_dir_all(root).unwrap();
}
