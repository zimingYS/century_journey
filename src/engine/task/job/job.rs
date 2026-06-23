use crate::engine::task::job::dependency::TaskDependency;
use crate::engine::task::job::id::TaskId;
use crate::engine::task::job::result::TaskResult;
use crate::engine::task::job::state::TaskState;
use crate::engine::task::scheduler::priority::TaskPriority;
use crate::engine::task::worker::kind::WorkerKind;
use std::time::Instant;

/// 可执行任务作业单元
/// 封装单个任务的完整元信息、执行状态与实际执行逻辑，承载任务的全生命周期管理
pub struct TaskJob {
    /// 任务唯一标识 ID
    pub id: TaskId,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 任务当前执行状态
    pub state: TaskState,
    /// 任务创建时间
    pub created_at: Instant,
    /// 任务开始执行时间，未启动时为 None
    pub started_at: Option<Instant>,
    /// 任务执行结束时间，未完成时为 None
    pub finished_at: Option<Instant>,
    /// 任务执行结果，未完成时为 None
    pub result: Option<TaskResult>,
    /// 任务依赖关系集合
    pub dependencies: TaskDependency,
    /// 执行该任务的工作线程类型
    pub worker_kind: WorkerKind,
    /// 内部持有的可执行任务闭包，执行后会被消费移除
    task: Option<Box<dyn FnOnce() -> TaskResult + Send>>,
}

impl TaskJob {
    /// 创建新的任务作业实例
    /// 自动生成任务 ID，初始状态设为「已创建」，默认使用 CPU 类型工作线程
    pub fn new(priority: TaskPriority, task: impl FnOnce() -> TaskResult + Send + 'static) -> Self {
        Self {
            id: TaskId::new(),
            priority,
            state: TaskState::Created,
            created_at: Instant::now(),
            started_at: None,
            finished_at: None,
            result: None,
            dependencies: TaskDependency::new(),
            worker_kind: WorkerKind::Cpu,
            task: Some(Box::new(task)),
        }
    }

    /// 指定任务对应的工作线程类型
    /// 链式调用方法，设置完成后返回修改后的任务实例
    pub fn with_worker_kind(mut self, kind: WorkerKind) -> Self {
        self.worker_kind = kind;
        self
    }

    /// 执行任务（由工作线程 Worker 调用，消费自身）
    /// 会消费内部持有的任务闭包，更新任务运行状态、起止时间与执行结果
    /// 若任务闭包已被消费，则返回「无有效任务」的失败结果
    /// 执行完成后返回自身，用于后续状态查询与结果回收
    pub fn execute(mut self) -> TaskJob {
        self.state = TaskState::Running;
        self.started_at = Some(Instant::now());

        let result = self
            .task
            .take()
            .map(|f| f())
            .unwrap_or(TaskResult::Failed("no task".into()));

        self.finished_at = Some(Instant::now());
        self.state = match &result {
            TaskResult::Success => TaskState::Completed,
            TaskResult::Failed(_) => TaskState::Failed,
        };
        self.result = Some(result);
        self.task = None;
        self
    }
}
