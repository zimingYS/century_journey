//! 覆盖合并后的 PNG 文件可读性与解码结果校验。

use super::super::ContentCheckReport;
use crate::engine::asset::{AssetFiles, AssetResolver};

/// 校验覆盖合并后的所有 PNG 文件均可读取和解码。
pub(in crate::content::validation) fn validate_textures(
    _resolver: &AssetResolver,
    files: &AssetFiles<'_>,
    report: &mut ContentCheckReport,
) {
    for directory in ["textures/blocks", "textures/items"] {
        let textures = files.resolved_files(directory, "png");
        report.checked_files += textures.len();
        for texture in textures {
            match std::fs::read(&texture.full_path) {
                Ok(bytes) => match image::load_from_memory(&bytes) {
                    Ok(image) if image.width() > 0 && image.height() > 0 => {}
                    Ok(_) => report.errors.push(format!(
                        "{}:image.dimensions: width and height must be positive",
                        texture.full_path.display()
                    )),
                    Err(error) => report.errors.push(format!(
                        "{}:image.data: cannot decode PNG: {error}",
                        texture.full_path.display()
                    )),
                },
                Err(error) => report.errors.push(format!(
                    "{}:image.data: cannot read texture: {error}",
                    texture.full_path.display()
                )),
            }
        }
    }
}
