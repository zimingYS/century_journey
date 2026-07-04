use super::player_model::PlayerSaveData;
use bincode::Options;
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use std::io::{Read, Write};

/// 序列化并压缩写入玩家数据
pub fn write_player_data(data: &PlayerSaveData, path: &std::path::Path) -> Result<(), String> {
    let serialized = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .serialize(data)
        .map_err(|e| format!("bincode serialize: {e}"))?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder
        .write_all(&serialized)
        .map_err(|e| format!("gzip write: {e}"))?;
    let compressed = encoder.finish().map_err(|e| format!("gzip finish: {e}"))?;
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(path, compressed).map_err(|e| format!("file write: {e}"))?;
    Ok(())
}

/// 读取并解压反序列化玩家数据
pub fn read_player_data(path: &std::path::Path) -> Result<PlayerSaveData, String> {
    let compressed = std::fs::read(path).map_err(|e| format!("file read: {e}"))?;
    let mut decoder = GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("gzip decompress: {e}"))?;
    let data: PlayerSaveData = bincode::DefaultOptions::new()
        .with_varint_encoding()
        .deserialize(&decompressed)
        .map_err(|e| format!("bincode deserialize: {e}"))?;
    Ok(data)
}

/// 获取玩家存档文件路径
pub fn player_save_path(world_name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from("saves")
        .join(world_name)
        .join("players")
        .join("singleplayer.dat")
}
