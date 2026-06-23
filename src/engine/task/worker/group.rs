use crate::engine::task::worker::kind::WorkerKind;

/// WorkerGroup — 按类型管理 Worker
pub struct WorkerGroup {
    pub kind: WorkerKind,
    pub count: usize,
}

impl WorkerGroup {
    pub fn new(kind: WorkerKind, count: usize) -> Self {
        Self { kind, count }
    }

    pub fn cpu(count: usize) -> Self {
        Self::new(WorkerKind::Cpu, count)
    }

    pub fn io(count: usize) -> Self {
        Self::new(WorkerKind::Io, count)
    }
}
