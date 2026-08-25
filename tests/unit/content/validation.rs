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
    let override_file = root.join("definitions/loot/blocks/century_journey/stone.json");
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
        error.contains("definitions/loot/blocks/century_journey/stone:entries[0].item")
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

#[test]
fn locale_key_mismatch_is_part_of_global_content_validation() {
    let root = std::env::temp_dir().join(format!(
        "century_journey_content_locale_{}",
        std::process::id()
    ));
    let override_file = root.join("locales/en-US.toml");
    std::fs::create_dir_all(override_file.parent().unwrap()).unwrap();
    // 覆盖 en-US：缺少大量 zh-CN 键，多出 menu.extra；键差异按排序逐条报告并截断。
    std::fs::write(
        &override_file,
        "language = \"en-US\"\nnative-name = \"English\"\n\n[menu]\nextra = \"Extra\"\n",
    )
    .unwrap();
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
            .any(|error| { error.contains("locales/en-US:locale.keys: missing key common.on") })
    );
    assert!(
        compilation
            .report
            .errors
            .iter()
            .any(|error| { error.contains("locales/en-US:locale.keys: extra key menu.extra") })
    );
    // 键差异超过逐条报告上限时应汇总剩余数量。
    assert!(compilation.report.errors.iter().any(|error| {
        error.contains("locales/en-US:locale.keys: ... and") && error.contains("more missing keys")
    }));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn missing_fallback_locale_is_reported() {
    let root = std::env::temp_dir().join(format!(
        "century_journey_content_locale_fallback_{}",
        std::process::id()
    ));
    let resolver = AssetResolver::new(&root);

    let compilation = compile_content(&resolver);

    assert!(compilation.report.errors.iter().any(|error| {
        error.contains("locales:locale.language: missing fallback language zh-CN")
    }));
}
