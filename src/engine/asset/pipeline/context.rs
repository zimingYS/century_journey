use crate::engine::asset::identifier::AssetId;
use crate::engine::asset::pipeline::request::AssetRequest;
use crate::engine::asset::state::{AssetMetadata, AssetState};
use std::collections::HashMap;

/// Pipeline 上下文
///
/// 在 Pipeline 各 Stage 之间传递，每个 Stage 只允许读取/修改 Context。
/// Stage 之间禁止直接通信。
pub struct AssetPipelineContext {
    /// 原始请求
    pub request: AssetRequest,
    /// 资源元数据（Stage 执行过程中逐步填充）
    pub metadata: AssetMetadata,

    /// 解析后的文件路径
    pub resolved_path: Option<String>,
    /// 原始字节数据（Source 阶段填充）
    pub raw_bytes: Option<Vec<u8>>,
    /// 反序列化/加载后的对象键（暂时用 type_id 字符串标识类型）
    pub loaded_type: Option<String>,

    /// 依赖关系
    pub dependencies: Vec<AssetId>,
    /// 诊断信息
    pub diagnostics: Vec<String>,
    /// 扩展数据（Stage 间传递临时数据）
    pub extensions: HashMap<String, String>,
}

impl AssetPipelineContext {
    /// 从 AssetRequest 创建初始 Context
    pub fn new(request: AssetRequest) -> Self {
        let mut metadata = AssetMetadata::new(request.id.clone(), request.source.clone());
        metadata.asset_type = request.asset_type.clone();
        Self {
            request,
            metadata,
            resolved_path: None,
            raw_bytes: None,
            loaded_type: None,
            dependencies: Vec::new(),
            diagnostics: Vec::new(),
            extensions: HashMap::new(),
        }
    }

    /// 记录诊断信息
    pub fn diagnose(&mut self, msg: impl Into<String>) {
        self.diagnostics.push(msg.into());
    }

    /// 标记失败
    pub fn fail(&mut self, error: impl Into<String>) {
        let err = error.into();
        self.metadata.state = AssetState::Failed;
        self.metadata.last_error = Some(err.clone());
        self.diagnostics.push(format!("FAIL: {}", err));
    }

    /// 是否已经失败
    pub fn is_failed(&self) -> bool {
        matches!(self.metadata.state, AssetState::Failed)
    }
}
