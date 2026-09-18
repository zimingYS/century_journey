//! 架构边界测试：读源码做文本断言

use std::fs;
use std::path::{Path, PathBuf};

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("应位于 crates/<crate> 之下")
        .to_path_buf()
}

fn src_files(crate_name: &str) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    collect(&crates_dir().join(crate_name).join("src"), &mut files);
    files
}

fn collect(dir: &Path, files: &mut Vec<(PathBuf, String)>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let text = fs::read_to_string(&path).unwrap_or_default();
            files.push((path, text));
        }
    }
}

fn assert_absent(crate_name: &str, forbidden: &[&str]) {
    for (path, text) in src_files(crate_name) {
        for needle in forbidden {
            assert!(
                !text.contains(needle),
                "{} 中出现了 `{needle}` —— 违反架构纪律",
                path.display()
            );
        }
    }
}

fn references_module(text: &str, module: &str) -> bool {
    let needle = format!("crate::{module}");

    let mut from = 0;
    while let Some(index) = text[from..].find(&needle) {
        let end = from + index + needle.len();
        let follows_ident = text[end..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');

        if !follows_ident {
            return true;
        }

        from = end;
    }

    false
}

#[test]
fn core_stays_engine_free() {
    assert_absent("core", &["bevy::"]);
}

#[test]
fn sim_does_not_depend_on_engine_or_storage() {
    assert_absent("sim", &["bevy::", "cj_world", "cj_runtime"]);
}

#[test]
fn world_does_not_depend_on_engine_or_upper_layers() {
    assert_absent("world", &["bevy::", "cj_sim", "cj_runtime"]);
}

#[test]
fn render_does_not_depend_on_sim_or_runtime() {
    assert_absent("render", &["cj_sim", "cj_runtime"]);
}

#[test]
fn runtime_does_not_depend_on_render_or_game() {
    assert_absent("runtime", &["cj_render", "cj_game"]);
}

#[test]
fn meshing_layers_stay_bevy_free() {
    for name in ["input.rs", "meshing.rs"] {
        let path = crates_dir().join("render").join("src").join(name);
        let text = fs::read_to_string(&path).expect("render 源码应存在");

        assert!(
            !text.contains("bevy::"),
            "{} 属于 render 的 L0/L1，不得依赖 bevy —— 否则纯算法失去可单测性",
            path.display()
        );
    }
}

#[test]
fn sim_domains_do_not_reference_each_other() {
    const DOMAINS: [&str; 7] = [
        "worldgen",
        "geo",
        "hydro",
        "climate",
        "eco",
        "structure",
        "event",
    ];

    for domain in DOMAINS {
        let path = crates_dir()
            .join("sim")
            .join("src")
            .join(format!("{domain}.rs"));

        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };

        for other in DOMAINS {
            if other == domain {
                continue;
            }

            assert!(
                !references_module(&text, other),
                "{} 引用了另一个领域 `crate::{other}` —— 领域间只能通过「场」交换数据",
                path.display()
            );
        }
    }
}
