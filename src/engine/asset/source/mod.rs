/// 资源来源 trait。
///
/// 抽象资源数据的获取方式。
/// V1 仅实现文件系统；后续可扩展 MemorySource、NetworkSource 等。
pub trait AssetSource: Send + Sync + 'static {
    /// 读取资源原始字节。
    fn read_bytes(&self, path: &str) -> Result<Vec<u8>, String>;
}

pub mod filesystem;
