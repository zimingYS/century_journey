/// 任务作用域执行器
/// 提供任务执行的作用域封装，以统一的作用域语义承载任务逻辑的执行
pub struct TaskScope;

impl TaskScope {
    /// 开启任务作用域并执行指定任务
    /// 在当前作用域内运行传入的一次性闭包，闭包执行完成后方法返回
    pub fn scope<F>(f: F)
    where
        F: FnOnce(),
    {
        f()
    }
}
