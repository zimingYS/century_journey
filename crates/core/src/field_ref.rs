//! 可观测场引用（采样接口输入，见 §6.8）
//!
//! 必须放在 core，否则 content 与 sim 会形成依赖环。

/// 可观测的场
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum FieldRef {
    Temperature,
    Humidity,
    Wind,
    WaterLevel,
    FlowVelocity,
    SoilTexture,
    SoilFertility,
    SoilDepth,
    SoilSalinity,
    Snowpack,
    IceThickness,
    Vegetation,
    // Population(SpeciesId),
    WaterTable,
}

/// 采样结果
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SampleValue {
    Scalar(f32),
    Vector([f32; 2]),
    /// 该位置无此场
    None,
}
