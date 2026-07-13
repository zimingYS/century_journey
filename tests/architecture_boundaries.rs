use std::fs;
use std::path::{Path, PathBuf};

struct LayerRule {
    directory: &'static str,
    forbidden_layers: &'static [&'static str],
}

const LAYER_RULES: &[LayerRule] = &[
    LayerRule {
        directory: "engine",
        forbidden_layers: &["app", "client", "content", "game"],
    },
    LayerRule {
        directory: "shared",
        forbidden_layers: &["app", "client", "content", "engine", "game"],
    },
    LayerRule {
        directory: "content",
        forbidden_layers: &["app", "client", "game"],
    },
    LayerRule {
        directory: "game",
        forbidden_layers: &["app", "client"],
    },
];

#[test]
fn source_layers_only_reference_allowed_dependencies() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for rule in LAYER_RULES {
        let layer_root = source_root.join(rule.directory);
        for file in rust_source_files(&layer_root) {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("无法读取 {}: {error}", file.display()));

            for (line_index, line) in source.lines().enumerate() {
                for forbidden in rule.forbidden_layers {
                    let crate_path = format!("crate::{forbidden}");
                    let public_path = format!("century_journey::{forbidden}");
                    if line.contains(&crate_path) || line.contains(&public_path) {
                        let relative = file.strip_prefix(&source_root).unwrap_or(&file);
                        violations.push(format!(
                            "{}:{}: {} 层禁止依赖 {} 层: {}",
                            relative.display(),
                            line_index + 1,
                            rule.directory,
                            forbidden,
                            line.trim()
                        ));
                    }
                }
            }
        }
    }

    let separator = char::from(10).to_string();
    let details = violations.join(&separator);
    assert!(violations.is_empty(), "检测到非法层级依赖: {details}");
}

fn rust_source_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rust_source_files(root, &mut files);
    files.sort();
    files
}

fn collect_rust_source_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("无法读取目录 {}: {error}", directory.display()));

    for entry in entries {
        let path = entry.expect("无法读取源码目录项").path();
        if path.is_dir() {
            collect_rust_source_files(&path, files);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
}
