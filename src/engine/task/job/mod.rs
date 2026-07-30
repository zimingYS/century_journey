//! 汇总任务标识、句柄与结果契约。

pub mod handle;
pub mod id;
pub mod result;

pub use handle::TaskHandle;
pub use id::TaskId;
pub use result::TaskResult;
