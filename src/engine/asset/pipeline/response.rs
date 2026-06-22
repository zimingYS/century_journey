use bevy::prelude::*;

/// 资源加载响应
///
/// Pipeline 执行完毕后返回。包含加载结果和诊断信息。
pub struct AssetResponse {
    /// 是否加载成功
    pub success: bool,
    /// 类型擦除的 Handle（成功时填充）
    pub handle: Option<UntypedHandle>,
    /// 资源元数据
    pub metadata_short: AssetResponseMetadata,
    /// 诊断日志
    pub diagnostics: Vec<String>,
}

/// 精简的响应元数据（给调用方参考）
pub struct AssetResponseMetadata {
    pub id: String,
    pub asset_type: String,
    pub load_time_ms: f64,
}

impl AssetResponse {
    /// 创建成功响应
    pub fn success(handle: UntypedHandle, meta: AssetResponseMetadata) -> Self {
        Self {
            success: true,
            handle: Some(handle),
            metadata_short: meta,
            diagnostics: Vec::new(),
        }
    }

    /// 创建成功响应（无 Handle，由 Manager 从 Cache 获取）
    pub fn success_empty(meta: AssetResponseMetadata) -> Self {
        Self {
            success: true,
            handle: None,
            metadata_short: meta,
            diagnostics: Vec::new(),
        }
    }

    /// 创建失败响应
    pub fn failed(error: &str, meta: AssetResponseMetadata) -> Self {
        Self {
            success: false,
            handle: None,
            metadata_short: meta,
            diagnostics: vec![error.to_string()],
        }
    }
}
