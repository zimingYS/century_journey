# 贡献指南

感谢你的兴趣。本项目目前处于**早期开发阶段**。

## 先开 Issue，再动手

- **Bug 修复**：先开 Bug 报告，确认可复现
- **新功能**：先开功能建议
- **文档 / 内容修正**：可直接提 PR

## 设计冻结说明

项目的**核心设计已冻结**（世界模拟、玩法结构、架构），当前阶段是**原型验证**。

因此：

- 涉及核心系统的大改动，请**先讨论再动手**
- 小修小补、内容扩展、文档、工具链改进——欢迎直接提 PR

## 开发环境

### 前置

- Rust **stable**（`edition = "2024"` 需要 **1.85+**）
- Git

### 构建与运行

```bash
git clone https://github.com/zimingYS/century_journey.git
cd century_journey
cargo build --release
cargo run --release
```

### 提交前自检

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

CI 跑同样的三项，本地过了一般就能过。

## 代码规范

### 分层依赖（硬约束）

```
app → game → runtime → sim / world → content → core
```

**禁止**：

- `sim` 依赖 Bevy / ECS / 渲染 / UI
- `world` 依赖 `runtime` / `render` / `game`
- `render` 依赖 `sim` / `runtime`
- `content` 依赖 `sim` / `world`
- `core` 依赖任何上层

破坏依赖方向会导致模拟层无法独立测试、无法批处理、无法 GPU 平移。

### 模拟算子约定

算子应尽量接近：

```rust
fn update(input: &Input, params: &Params, dt: f32) -> Output{}
```

不读全局状态、不读 ECS、不依赖全局时间。

### 提交信息

采用 Conventional Commits：

```
feat: 新增积雪融化速率
fix: 修正河流基流在旱季归零
refactor: 拆分 world 的 delta 与 integral
docs: 补充观测系统说明
chore: 升级依赖
```

## 许可与贡献

本项目采用**源可用（Source-Available）**&#8203;模式：

- 提交代码即表示你同意你的贡献以本项目的许可（PolyForm Noncommercial 1.0.0）分发
- 若你希望保留自己贡献的独立版权用于其他用途，请在 PR 中说明

详见 [LICENSE](LICENSE) 与 [MOD-POLICY.md](MOD-POLICY.md)。
