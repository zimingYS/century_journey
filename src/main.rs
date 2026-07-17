use century_journey::app;
use century_journey::content::validation::check_content;
use century_journey::engine::asset::AssetResolver;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--check-content")) {
        let root = args
            .next()
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("CJ_ASSET_ROOT").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("assets"));
        if let Some(extra) = args.next() {
            anyhow::bail!(
                "unexpected argument for --check-content: {}",
                extra.to_string_lossy()
            );
        }
        let overrides = std::env::var_os("CJ_CONTENT_OVERRIDES")
            .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
            .unwrap_or_default();
        let resolver = AssetResolver::with_content_overrides(root, overrides);
        let report = check_content(&resolver);
        if report.is_valid() {
            println!("content check passed: {} files", report.checked_files);
            return Ok(());
        }
        for error in &report.errors {
            eprintln!("content error: {error}");
        }
        anyhow::bail!(
            "content check failed: {} error(s) in {} file(s)",
            report.errors.len(),
            report.checked_files
        );
    }
    app::launch()
}
