# Century Journey 仓库协作规则

本文件适用于整个仓库。开始修改代码前，必须先阅读：

1. `docs/CODE_STYLE.md`：代码、注释、模块、测试和验证规范。
2. `docs/PROJECT_STRUCTURE.md`：分层边界与模块职责。
3. 修改数据驱动内容时，额外阅读 `docs/content-format.md`。

## 强制要求

- 源码、配置和文档统一使用 UTF-8；禁止提交乱码或依赖本机编码的文本。
- 新增或修改的模块、公共类型、公共函数以及复杂算法必须编写简洁中文注释。
- 注释解释职责、约束、原因和边界条件，不逐行翻译显而易见的代码。
- Rust 标识符使用英文并遵循 Rust 命名习惯；面向玩家的正式文本使用中文。
- 优先通过数据、组件和小模块扩展功能，不在现有大文件中继续堆叠无关职责。
- 单个生产代码文件达到 500 行时必须评估拆分；超过 800 行原则上不得继续增加功能。
- 所有测试源码统一放在根目录 `tests/`。白盒单元测试放在 `tests/unit/`，集成测试按领域放在 `tests/` 子目录。
- `src/` 中只允许保留指向 `tests/unit/` 的 `#[cfg(test)] #[path = "..."] mod tests;` 声明，不直接编写测试实现。
- 不得为了迁移测试而把内部实现无意义地改成 `pub`；白盒测试通过外部测试模块保持原有可见性。
- 修改 Bevy 系统时必须明确其调度阶段、顺序约束和时间源；游戏规则进入固定步，纯表现进入渲染帧。
- 不得用新增全局 `allow` 掩盖 Clippy 或编译问题；确有必要时只做最小范围豁免并写明原因。

## 完成前检查

```text
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
```

涉及内容资产时还必须执行：

```text
cargo run --locked -- --check-content assets
```
