//! 提供带格式版本的数据驱动 JSON 加载入口。

use crate::engine::asset::AssetFiles;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// 当前数据驱动内容文件支持的格式版本。
pub const CONTENT_FORMAT_VERSION: u32 = 1;

/// 所有 definitions JSON 的统一版本外壳。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Versioned<T> {
    pub format_version: u32,
    #[serde(flatten)]
    pub content: T,
}

impl<T> Versioned<T> {
    /// 校验格式版本并提取当前版本的数据。
    pub fn into_current(self, asset_path: &str) -> Result<T, String> {
        if self.format_version != CONTENT_FORMAT_VERSION {
            return Err(format!(
                "{asset_path}:format_version: unsupported value {}, expected {}",
                self.format_version, CONTENT_FORMAT_VERSION
            ));
        }
        Ok(self.content)
    }
}

/// 加载目录中格式版本匹配的全部 JSON 定义。
pub fn load_versioned_json_dir<T: DeserializeOwned>(
    files: &AssetFiles<'_>,
    dir_path: &str,
) -> Vec<(String, T)> {
    versioned_json_dir_results(files, dir_path)
        .into_iter()
        .filter_map(|result| match result {
            Ok(value) => Some(value),
            Err(error) => {
                log::warn!("[内容] {error}");
                None
            }
        })
        .collect()
}

/// 逐文件返回带路径上下文的版本化 JSON 加载结果。
pub fn versioned_json_dir_results<T: DeserializeOwned>(
    files: &AssetFiles<'_>,
    dir_path: &str,
) -> Vec<Result<(String, T), String>> {
    files
        .read_json_dir_results::<Versioned<T>>(dir_path)
        .into_iter()
        .map(|result| {
            let (asset_path, versioned) = result?;
            let content = versioned.into_current(&asset_path)?;
            Ok((asset_path, content))
        })
        .collect()
}
