# CenturyJourney（世纪之旅）

一个必须读懂地质与气候规律，才能长期定居的体素生存游戏。Rust + Bevy 实现。

## 状态

**早期原型验证阶段**。核心设计已冻结（世界模拟、玩法结构、架构），P0 最小渲染链路接近完成；代码以源可用形式公开（许可见下）。

## 前置要求

| 依赖 | 版本 | 说明 |
|---|---|---|
| Rust | **stable 1.85+**（edition 2024） | 用 [rustup](https://rustup.rs) 安装即可；仓库已固定 `rust-toolchain.toml` |
| Git | 任意近期版本 | — |

Linux 构建需系统库（Windows / macOS 无额外要求）：

```bash
sudo apt-get install pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
```

## 获取与构建

```bash
git clone https://github.com/zimingYS/century_journey.git
cd century_journey

# 调试版（开发迭代用）
cargo build
cargo run

# 发布版（游玩用）
cargo build --release
cargo run --release
```

首次构建需要编译 Bevy 及依赖，耗时较长；之后增量构建很快。

## 启动与操作

启动后会生成初始地形（约 289 个区块，0.4 秒量级）并进入第一人称视角。

| 按键 | 作用 |
|---|---|
| `W` `A` `S` `D` | 移动 |
| 鼠标 | 视角 |
| `空格` | 跳跃（飞行模式：上升） |
| `左 Ctrl` | 飞行模式：下降 |
| `左 Shift` | 冲刺 |
| `F` | 切换 步行 / 飞行 模式 |
| `Esc` | 释放 / 捕获鼠标光标 |

## 测试与检查

提交前请跑（与 CI 一致）：

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
```

CI 对每个 PR 自动执行上述检查，另有依赖审查（dependency review）与安全审计（rustsec audit）。

## 项目结构

```
crates/
├── core/      基础类型：体素、坐标、常量、RNG、方块 id（零依赖）
├── sim/       纯模拟：世界生成与领域算子（禁 bevy / world）
├── world/     存储层：区块、玩家改动 Delta、自然积分量（禁 bevy）
├── runtime/   模拟侧唯一接触 ECS：世界数据、生成装配、脏区跟踪
├── render/    渲染：快照 → 网格 → bevy（禁 sim / runtime）
├── game/      玩法：输入 → 交互 → 玩家 → 玩法域 → UI
└── app/       启动与组装（唯一 main）
```

架构细节见 **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)**，各模块契约见 **[docs/modules/](docs/modules/)**。

## 文档

| 文档 | 内容 |
|---|---|
| [ROADMAP.md](ROADMAP.md) | 开发路线图：P0-P6 阶段、完成标准、当前位置 |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | 系统架构：crate 分层、依赖纪律、数据流、插件组装 |
| [docs/modules/](docs/modules/) | 模块设计文档（每 crate 一份：职责 / API 契约 / 状态） |
| [docs/RELEASING.md](docs/RELEASING.md) | 版本与发布流程（含多平台发包） |
| [GOVERNANCE.md](GOVERNANCE.md) | 项目治理：决策流程、权限、合规 |

## 贡献

请先读 [CONTRIBUTING.md](CONTRIBUTING.md)——本项目**核心设计已冻结**，大改动请先开 Issue 讨论（设计提案模板）；小修复、文档、工具链改进欢迎直接提 PR。决策与治理见 [GOVERNANCE.md](GOVERNANCE.md)，MOD 生态规划见 [MOD-POLICY.md](MOD-POLICY.md)。

## 许可

双轨许可（PolyForm Noncommercial）：

- **代码**：[PolyForm Noncommercial License 1.0.0](LICENSE)（可自由学习、修改、非商业使用）
- **资产**（美术 / 音频 / 文案）：保留所有权利，见 LICENSE 附录
