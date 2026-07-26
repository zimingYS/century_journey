use century_journey::app;
use century_journey::content::validation::check_content;
use century_journey::engine::asset::AssetResolver;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    // 获取命令行参数，并跳过第一个参数
    let mut args = std::env::args_os().skip(1);
    // 使用"--check-content"参数进行资源目录检查
    if args.next().as_deref() == Some(std::ffi::OsStr::new("--check-content")) {
        // 获取资源根目录
        let root = args
            .next()
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("CJ_ASSET_ROOT").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("assets"));

        // 检查是否还有额外参数
        if let Some(extra) = args.next() {
            anyhow::bail!(
                "unexpected argument for --check-content: {}",
                extra.to_string_lossy()
            );
        }

        // 从环境变量读取额外内容覆盖路径
        let overrides = std::env::var_os("CJ_CONTENT_OVERRIDES")
            .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
            .unwrap_or_default();
        // 创建资源解析器
        let resolver = AssetResolver::with_content_overrides(root, overrides);
        // 执行内容检查
        let report = check_content(&resolver);

        // 检查是否通过
        if report.is_valid() {
            println!("content check passed: {} files", report.checked_files);
            // 检查通过则直接退出
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

    // 没有输入则正常进入游戏
    app::launch()
}
