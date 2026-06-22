/// 资源后处理器 trait
///
/// 在 Loader 完成反序列化后执行。
/// 支持链式调用。
pub trait AssetProcessor: Send + Sync + 'static {
    /// 处理器名称
    fn name(&self) -> &str;

    /// 处理原始/中间数据
    fn process(&self, data: &[u8]) -> Result<Vec<u8>, String>;
}

/// 处理器链 — 按顺序执行多个 Processor
pub struct ProcessorChain {
    processors: Vec<Box<dyn AssetProcessor>>,
}

impl ProcessorChain {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
        }
    }

    pub fn add(mut self, processor: impl AssetProcessor) -> Self {
        self.processors.push(Box::new(processor));
        self
    }

    pub fn process_all(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let mut current = data.to_vec();
        for p in &self.processors {
            current = p
                .process(&current)
                .map_err(|e| format!("[{}] {}", p.name(), e))?;
        }
        Ok(current)
    }

    pub fn is_empty(&self) -> bool {
        self.processors.is_empty()
    }
}

impl Default for ProcessorChain {
    fn default() -> Self {
        Self::new()
    }
}
