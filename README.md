# Century Journey
# 世纪之旅

<div style="text-align: center;">

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-Game%20Engine-blue)
![License](https://img.shields.io/badge/Code-MPL--2.0-green)
![Assets](https://img.shields.io/badge/Assets-CJAL%20v1.0-red)
![Platform](https://img.shields.io/badge/Platform-Windows%20tested-lightgrey)

**A modern voxel sandbox game built with Rust and Bevy.**

**使用 Rust 和 Bevy 开发的现代体素沙盒游戏。**

</div>

---

## Overview
## 项目概述

Century Journey is an early-stage voxel sandbox prototype focused on performance, moddability, and long-term extensibility.

《世纪之旅》是一款处于早期开发阶段的体素沙盒原型，重点关注性能、可模组性和长期可扩展性。

The current content pipeline is data-driven for blocks, items, recipes, loot tables, tags, and selected biome definitions. These formats are still evolving and are not yet a stable public modding API.

当前内容管线已对方块、物品、配方、掉落表、标签和部分生物群系定义采用数据驱动设计。这些格式仍在演进，尚未形成稳定的公共模组 API。

Project status: **early active development**. The current scope is a local single-player voxel sandbox technical prototype. Server mode, editor mode, multiplayer, and network synchronization are planned but not implemented yet.

项目状态：**早期积极开发阶段**。当前定位是本地单机 voxel sandbox 技术原型；Server 模式、Editor 模式、Multiplayer 与网络同步仍处于规划阶段，尚未实现为可用能力。

---

## Features
## 特性

- Built with Rust.
  基于 Rust 开发。
- Powered by the Bevy game engine.
  由 Bevy 游戏引擎驱动。
- Procedurally generated voxel world with chunk streaming.
  支持区块流式加载的程序化体素世界。
- Data-driven blocks, items, recipes, loot tables, and tags.
  数据驱动的方块、物品、配方、掉落表与标签。
- Mod API, Mod SDK, and resource-pack support are planned.
  Mod API、Mod SDK 与资源包支持仍处于规划阶段。
- Complete day and night cycle.
  完整的昼夜循环系统。
- Dynamic world simulation is planned.
  动态世界模拟处于规划阶段。
- Dynamic weather is planned.
  动态天气系统处于规划阶段。
- Ecology and seasonal mechanics are planned.
  生态与季节机制处于规划阶段。
- Cross-platform multiplayer is planned.
  跨平台多人联机处于规划阶段。
- Native cross-platform support is planned for Windows, Linux, and macOS.
  计划支持 Windows、Linux 与 macOS 原生跨平台运行。

---

## Architecture
## 架构

The project uses a layered architecture so systems can evolve independently and remain easier to maintain. At this stage, Content and Game are used by the local client prototype; Server, Protocol, and Editor are planned architecture boundaries.

项目采用分层架构，使各系统能够独立演进，并保持较好地可维护性。现阶段 Content 与 Game 主要服务于本地客户端原型；Server、Protocol 与 Editor 是规划中的架构边界。

当前源码职责、依赖方向和中文注释规范见
[项目结构文档](docs/PROJECT_STRUCTURE.md)。

```text
App
|-- application flow
`-- plugin assembly

Client
|-- input and UI
|-- renderer and sky
`-- audio, particles, and effects

Game
|-- world and player
|-- inventory and crafting
`-- gameplay rules

Content
|-- blocks, items, and biomes
`-- recipes, loot, and tags

Shared
`-- shared data types

Engine
|-- asset pipeline
`-- task facade

Planned boundaries
|-- Editor
|-- Protocol
`-- Server
```

---

## Modding Support
## 模组支持

Century Journey is being structured with future modding support in mind, but it does not currently provide a stable public Mod API, Mod SDK, or resource-pack compatibility contract.

《世纪之旅》的架构为未来模组支持预留了边界，但当前尚未提供稳定的公共 Mod API、Mod SDK 或资源包兼容性承诺。

Internal content formats may change without migration guarantees before the public modding interface is stabilized.

在公共模组接口稳定之前，内部内容格式可能发生变化，且暂不保证迁移兼容性。

---

## Building
## 构建

Install a complete Rust toolchain with Cargo before building the project.

构建项目之前，请先安装包含 Cargo 的完整 Rust 工具链。

1. Clone the repository.
   克隆本仓库。
2. Run `cargo build` to build the project.
   执行 `cargo build` 构建项目。
3. Run `cargo run` to start the application.
   执行 `cargo run` 启动应用。

---

## Contributing
## 贡献

Contributions are welcome. Focused issues and small pull requests are the easiest to review while the project is still evolving quickly.

欢迎提交贡献。在项目仍快速演进的阶段，聚焦的问题反馈和范围较小的 Pull Request 更容易被审阅。

Before submitting a pull request, please read these documents:

提交 Pull Request 前，请阅读以下文档：

- `CONTRIBUTING.md`
- `CODE_OF_CONDUCT.md`
- `SECURITY.md`

---

## Documentation
## 文档

Official documentation will be kept in this repository.

官方文档会统一维护在本仓库中。

Documentation is being expanded as the project stabilizes.

文档会随着项目稳定逐步补充。

---

## Development Roadmap
## 开发路线图

- [x] Chunk system.
  区块系统。
- [x] Block system.
  方块系统。
- [x] Inventory system.
  背包系统。
- [x] View model rendering.
  手持物品渲染。
- [x] Item system.
  物品系统。
- [ ] Entity system.
  实体系统。
- [ ] World generation 2.0.
  二代世界生成。
- [ ] Official Mod SDK.
  官方模组开发套件。
- [ ] Multiplayer.
  多人联机。
- [ ] Dedicated server.
  独立专用服务器。
- [ ] Steam release.
  Steam 正式发布。

---

## License
## 许可证

Century Journey uses multiple licenses depending on the project component.

《世纪之旅》根据项目组成部分使用不同的许可证。

- Repository license entry: [LICENSE.md](LICENSE.md).
  仓库许可证入口：[LICENSE.md](LICENSE.md)。
- Source code: [Mozilla Public License 2.0 (MPL-2.0)](LICENSES/MPL-2.0.txt).
  项目源码：Mozilla Public License 2.0（MPL-2.0）。
- Official game assets: [Century Journey Assets License (CJAL v1.0)](LICENSES/CJAL-1.0.md).
  官方游戏资源：Century Journey Assets License（CJAL v1.0）。
- Mod SDK and API: [MIT License](LICENSES/MOD-SDK-MIT.txt).
  模组 SDK 与 API：MIT License。
- Documentation: Creative Commons Attribution 4.0 International (CC BY 4.0).
  文档：知识共享署名 4.0 国际许可协议（CC BY 4.0）。
- Third-party resources: see [third-party notices](LICENSES/THIRD_PARTY_NOTICES.md).
  第三方资源：见[第三方声明](LICENSES/THIRD_PARTY_NOTICES.md)。

### Source Code
### 项目源码

The source code located in the `src/` directory is licensed under the Mozilla Public License 2.0 (MPL-2.0).

`src/` 目录中的项目源码遵循 Mozilla Public License 2.0（MPL-2.0）。

### Game Assets
### 游戏资源

Textures, models, sounds, music, icons, animations, shaders, and other official game assets are licensed under the Century Journey Assets License (CJAL v1.0).

纹理、模型、音效、音乐、图标、动画、着色器及其他官方游戏资源遵循 Century Journey Assets License（CJAL v1.0）。

Unless explicitly permitted by the asset license, official assets may not be used in other games or commercial products.

除非资源许可证明确允许，官方资源不得用于其他游戏或商业产品。

### Mod SDK
### 模组开发套件

Code explicitly released as part of the planned official Mod SDK or public Mod API will use the MIT License. No stable SDK or public API is available yet.

未来明确作为官方 Mod SDK 或公共 Mod API 发布的代码将使用 MIT License；当前尚未提供稳定 SDK 或公共 API。

### Documentation
### 文档

Project documentation is licensed under Creative Commons Attribution 4.0 International (CC BY 4.0).

项目文档遵循知识共享署名 4.0 国际许可协议（CC BY 4.0）。

### Third-party Resources
### 第三方资源

Third-party resources keep their own licenses. See [third-party notices](LICENSES/THIRD_PARTY_NOTICES.md).

第三方资源遵循其各自许可证，详见[第三方声明](LICENSES/THIRD_PARTY_NOTICES.md)。

---

## Special Thanks
## 特别鸣谢

Thanks to the Rust community, the Bevy community, every contributor, and every player who supports the project.

感谢 Rust 社区、Bevy 社区、每一位贡献者，以及每一位支持本项目的玩家。

---

<div style="text-align: center;">

**Made with care using Rust.**

**用 Rust 用心构建。**

Century Journey (C) 2026 Contributors

《世纪之旅》 (C) 2026 全体贡献者

</div>
