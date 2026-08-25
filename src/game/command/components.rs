//! 定义指令系统跨层传递的消息。

use bevy::prelude::*;

/// 玩家提交的原始指令行（已剥离前导 '/'，不含聊天语义）。
#[derive(Message, Debug, Clone)]
pub struct GameCommandSubmitted {
    /// 剥离 '/' 后的原始指令文本。
    pub raw: String,
}

/// 指令执行结果回显，由客户端控制台 UI 消费展示。
#[derive(Message, Debug, Clone)]
pub struct CommandOutput {
    /// 面向玩家的反馈文本，已按当前语言完成本地化。
    pub text: String,
}
