//! 可观测场引用（采样接口的输入，见 §6.8）
//!
//! 注意：FieldRef / SampleValue 必须放在 core，
//! 否则 content（工具定义）与 sim（采样实现）会形成依赖环。

/// 有哪些可观测的场
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FieldRef {
    Temperature,
    Humidity,
    Wind,
    WaterLevel,
}

/// 采样结果的统一类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleValue {
    Scalar(f32),
    Vector([f32; 2]),
    /// 该位置无此场（如陆地采冰厚）
    None,
}