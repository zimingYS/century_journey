/// Worker 类型
/// Scheduler 自动将 Task 派发至对应类型的 Worker。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerKind {
    /// CPU 密集型（默认）
    Cpu,
    /// IO 密集型（文件读写/网络）
    Io,
}
