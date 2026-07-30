//! 内容定义加载统计和稳定标识符去重辅助。

use super::ContentCheckReport;
use crate::content::format::versioned_json_dir_results;
use crate::engine::asset::AssetFiles;
use std::collections::HashSet;

/// 加载一个定义目录，并把所有解析错误汇总到统一报告。
pub(super) fn load<T: serde::de::DeserializeOwned>(
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

/// 收集稳定标识符，同时报告由不同文件声明的重复标识符。
pub(super) fn unique_ids<'a>(
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
