//! 为小型持久化文档提供带格式头的命名 MessagePack 编解码。
//!
//! 本模块只描述通用容器格式，不了解玩家、世界或设置等业务字段。
//! 文档正文使用字段名编码，因此结构体新增默认字段或删除旧字段时无需复制版本结构体。

use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io::{Read, Write};

const HEADER_SIZE: usize = 8;

/// 将当前结构编码为带 magic、格式号和 gzip 压缩的命名 MessagePack 文档。
///
/// `format` 只在外层编码、压缩或加密方式变化时递增；业务字段演进交给 Serde 处理。
pub fn encode_named<T: Serialize>(
    magic: [u8; 4],
    format: u32,
    value: &T,
) -> Result<Vec<u8>, String> {
    let payload = rmp_serde::to_vec_named(value)
        .map_err(|error| format!("MessagePack 序列化失败: {error}"))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&payload)
        .map_err(|error| format!("gzip 写入失败: {error}"))?;
    let compressed = encoder
        .finish()
        .map_err(|error| format!("gzip 完成失败: {error}"))?;

    let mut document = Vec::with_capacity(HEADER_SIZE + compressed.len());
    document.extend_from_slice(&magic);
    document.extend_from_slice(&format.to_le_bytes());
    document.extend_from_slice(&compressed);
    Ok(document)
}

/// 解码指定 magic 和格式号的命名 MessagePack 文档。
///
/// 解压上限用于阻止损坏或恶意文件无限扩张内存；调用方应按文档领域选择合理容量。
pub fn decode_named<T: DeserializeOwned>(
    bytes: &[u8],
    expected_magic: [u8; 4],
    supported_format: u32,
    max_decompressed_bytes: usize,
) -> Result<T, String> {
    let (format, compressed) = split_header(bytes, expected_magic)?;
    if format != supported_format {
        return Err(format!(
            "不支持的文档格式 {format}，当前仅支持 {supported_format}"
        ));
    }
    let payload = decompress_limited(compressed, max_decompressed_bytes)?;
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(payload.as_slice()));
    let value = T::deserialize(&mut decoder)
        .map_err(|error| format!("MessagePack 反序列化失败: {error}"))?;
    if decoder.position() != payload.len() as u64 {
        return Err("MessagePack 文档包含尾随字节".into());
    }
    Ok(value)
}

/// 判断字节流是否使用指定文档 magic。
pub fn has_magic(bytes: &[u8], magic: [u8; 4]) -> bool {
    bytes.starts_with(&magic)
}

fn split_header(bytes: &[u8], expected_magic: [u8; 4]) -> Result<(u32, &[u8]), String> {
    if bytes.len() < HEADER_SIZE {
        return Err(format!("文档头不完整：至少需要 {HEADER_SIZE} 字节"));
    }
    if bytes[..4] != expected_magic {
        return Err("文档 magic 不匹配".to_string());
    }
    let format = u32::from_le_bytes(
        bytes[4..HEADER_SIZE]
            .try_into()
            .map_err(|_| "文档格式号不完整".to_string())?,
    );
    Ok((format, &bytes[HEADER_SIZE..]))
}

fn decompress_limited(bytes: &[u8], max_bytes: usize) -> Result<Vec<u8>, String> {
    let read_limit = max_bytes.saturating_add(1) as u64;
    let mut decoder = GzDecoder::new(bytes).take(read_limit);
    let mut payload = Vec::new();
    decoder
        .read_to_end(&mut payload)
        .map_err(|error| format!("gzip 解压失败: {error}"))?;
    if payload.len() > max_bytes {
        return Err(format!("文档解压后超过 {max_bytes} 字节上限"));
    }
    Ok(payload)
}

#[cfg(test)]
#[path = "../../tests/unit/engine/document.rs"]
mod tests;
