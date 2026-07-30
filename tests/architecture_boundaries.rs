//! 验证源码分层、模块规模和白盒测试目录等仓库级结构约束。

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_PRODUCTION_FILE_LINES: usize = 800;

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

#[test]
fn production_source_files_respect_hard_line_limit() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let violations = rust_source_files(&source_root)
        .into_iter()
        .filter_map(|file| {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("无法读取 {}: {error}", file.display()));
            let line_count = source.lines().count();
            (line_count > MAX_PRODUCTION_FILE_LINES).then(|| {
                let relative = file.strip_prefix(&source_root).unwrap_or(&file);
                format!("{}: {line_count} 行", relative.display())
            })
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "生产代码文件超过 {MAX_PRODUCTION_FILE_LINES} 行硬限制:\n{}",
        violations.join("\n")
    );
}

#[test]
fn every_source_module_has_module_documentation() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let violations = rust_source_files(&source_root)
        .into_iter()
        .filter_map(|file| {
            let source = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("无法读取 {}: {error}", file.display()));
            let module_documentation = source
                .lines()
                .skip_while(|line| line.trim().is_empty())
                .take_while(|line| line.trim_start().starts_with("//!"))
                .collect::<Vec<_>>()
                .join("\n");
            (module_documentation.is_empty() || !contains_chinese(&module_documentation)).then(
                || {
                    file.strip_prefix(&source_root)
                        .unwrap_or(&file)
                        .display()
                        .to_string()
                },
            )
        })
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "以下源码模块缺少文件级 `//!` 中文职责说明:\n{}",
        violations.join("\n")
    );
}

#[test]
fn public_source_items_have_chinese_documentation() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for file in rust_source_files(&source_root) {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("无法读取 {}: {error}", file.display()));
        let lines = source.lines().collect::<Vec<_>>();

        for (line_index, line) in lines.iter().enumerate() {
            let Some(item) = public_item_declaration(line) else {
                continue;
            };

            let mut cursor = line_index;
            while cursor > 0 {
                let previous = lines[cursor - 1].trim();
                if previous.is_empty() || previous.starts_with("#[") {
                    cursor -= 1;
                } else {
                    break;
                }
            }

            let mut documentation = Vec::new();
            while cursor > 0 {
                let previous = lines[cursor - 1].trim_start();
                if previous.starts_with("///") {
                    documentation.push(previous);
                    cursor -= 1;
                } else {
                    break;
                }
            }

            let documentation = documentation.join("\n");
            if documentation.is_empty() || !contains_chinese(&documentation) {
                let relative = file.strip_prefix(&source_root).unwrap_or(&file);
                violations.push(format!("{}:{}: {item}", relative.display(), line_index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "以下公共类型或函数缺少中文 `///` 职责说明:\n{}",
        violations.join("\n")
    );
}

#[test]
fn lint_exceptions_are_local_and_explained() {
    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();

    for file in rust_source_files(&source_root) {
        let source = fs::read_to_string(&file)
            .unwrap_or_else(|error| panic!("无法读取 {}: {error}", file.display()));
        let lines = source.lines().collect::<Vec<_>>();
        let relative = file.strip_prefix(&source_root).unwrap_or(&file);

        for (line_index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("#![allow(") {
                violations.push(format!(
                    "{}:{}: 禁止使用 crate 或模块级 allow",
                    relative.display(),
                    line_index + 1
                ));
                continue;
            }
            if !trimmed.starts_with("#[allow(") {
                continue;
            }

            let reason_found = lines[..line_index]
                .iter()
                .rev()
                .take(6)
                .take_while(|previous| {
                    let previous = previous.trim();
                    previous.is_empty() || previous.starts_with("//")
                })
                .any(|comment| comment.trim().starts_with("//") && contains_chinese(comment));
            if !reason_found {
                violations.push(format!(
                    "{}:{}: 局部 allow 缺少紧邻的中文原因说明",
                    relative.display(),
                    line_index + 1
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Clippy 或编译豁免不符合最小范围规则:\n{}",
        violations.join("\n")
    );
}

#[test]
fn white_box_tests_are_mirrored_and_declared_once() {
    let manifest_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_root = manifest_root.join("src");
    let unit_root = manifest_root.join("tests").join("unit");
    let mut declarations: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
    let mut violations = Vec::new();

    for source_file in rust_source_files(&source_root) {
        let source = fs::read_to_string(&source_file)
            .unwrap_or_else(|error| panic!("无法读取 {}: {error}", source_file.display()));
        if source.contains("mod tests {") {
            violations.push(format!(
                "{}: 测试实现不得内联在 src 中",
                source_file
                    .strip_prefix(&source_root)
                    .unwrap_or(&source_file)
                    .display()
            ));
        }

        for declared_path in declared_test_paths(&source) {
            let resolved = source_file
                .parent()
                .expect("源码文件必须有父目录")
                .join(&declared_path);
            if !resolved.is_file() {
                violations.push(format!(
                    "{}: 测试声明不存在: {}",
                    source_file
                        .strip_prefix(&source_root)
                        .unwrap_or(&source_file)
                        .display(),
                    declared_path.display()
                ));
                continue;
            }

            let canonical = fs::canonicalize(&resolved)
                .unwrap_or_else(|error| panic!("无法规范化 {}: {error}", resolved.display()));
            let source_relative = source_file
                .strip_prefix(&source_root)
                .expect("源码必须位于 src 下");
            let expected = fs::canonicalize(unit_root.join(source_relative)).unwrap_or_else(|_| {
                panic!(
                    "{} 的白盒测试应镜像到 tests/unit/{}",
                    source_relative.display(),
                    source_relative.display()
                )
            });
            if canonical != expected {
                violations.push(format!(
                    "{}: 白盒测试应位于 tests/unit/{}",
                    source_relative.display(),
                    source_relative.display()
                ));
            }
            declarations
                .entry(canonical)
                .or_default()
                .push(source_file.clone());
        }
    }

    let declared_files = declarations.keys().cloned().collect::<HashSet<_>>();
    for test_file in rust_source_files(&unit_root) {
        let canonical = fs::canonicalize(&test_file)
            .unwrap_or_else(|error| panic!("无法规范化 {}: {error}", test_file.display()));
        if !declared_files.contains(&canonical) {
            violations.push(format!(
                "tests/unit/{}: 没有对应的 src 测试声明",
                test_file
                    .strip_prefix(&unit_root)
                    .unwrap_or(&test_file)
                    .display()
            ));
        }
    }
    for (test_file, owners) in declarations {
        if owners.len() > 1 {
            violations.push(format!("{}: 被多个源码模块重复声明", test_file.display()));
        }
    }

    assert!(
        violations.is_empty(),
        "白盒测试目录结构不符合镜像规则:\n{}",
        violations.join("\n")
    );
}

fn declared_test_paths(source: &str) -> Vec<PathBuf> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let value = line.strip_prefix("#[path = \"")?.strip_suffix("\"]")?;
            Some(PathBuf::from(value))
        })
        .collect()
}

fn public_item_declaration(line: &str) -> Option<&str> {
    let line = line.trim_start();
    // 宏模板中的 `$name` 会在展开时继承调用处元数据，源码扫描无法可靠判断其文档。
    if line.contains('$') {
        return None;
    }
    let mut declaration = if let Some(rest) = line.strip_prefix("pub ") {
        rest
    } else if let Some(rest) = line.strip_prefix("pub(") {
        let visibility_end = rest.find(')')?;
        rest.get(visibility_end + 1..)?.trim_start()
    } else {
        return None;
    };

    for modifier in ["async ", "unsafe "] {
        if let Some(rest) = declaration.strip_prefix(modifier) {
            declaration = rest;
        }
    }
    if let Some(rest) = declaration.strip_prefix("const fn ") {
        declaration = rest;
        return Some(declaration.split(['(', '<']).next().unwrap_or(declaration));
    }

    for keyword in [
        "fn ", "struct ", "enum ", "trait ", "type ", "const ", "static ",
    ] {
        if let Some(rest) = declaration.strip_prefix(keyword) {
            return Some(
                rest.split(['(', '<', ':', '=', ' ', '{'])
                    .next()
                    .unwrap_or(rest),
            );
        }
    }
    None
}

fn contains_chinese(text: &str) -> bool {
    text.chars().any(|character| {
        matches!(
            character,
            '\u{3400}'..='\u{4dbf}' | '\u{4e00}'..='\u{9fff}' | '\u{f900}'..='\u{faff}'
        )
    })
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
