use crate::engine::task::job::id::TaskId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskHandle {
    id: TaskId,
}

impl TaskHandle {
    pub(crate) fn new(id: TaskId) -> Self {
        Self { id }
    }

    pub fn id(&self) -> TaskId {
        self.id
    }
}
