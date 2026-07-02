# Century Journey

<div align="center">

![Rust](https://img.shields.io/badge/Rust-2024-orange?logo=rust)
![Bevy](https://img.shields.io/badge/Bevy-Game%20Engine-blue)
![License](https://img.shields.io/badge/Code-MPL--2.0-green)
![Assets](https://img.shields.io/badge/Assets-CJAL%20v1.0-red)
![Platform](https://img.shields.io/badge/Platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey)

**A modern voxel sandbox game built with Rust and Bevy.**
**使用 Rust 与 Bevy 开发的新一代体素沙盒游戏。**

</div>

---

## ✨ Overview 项目概述
Century Journey is a modern voxel sandbox game focused on performance, moddability, and long-term extensibility.
Unlike traditional sandbox games, almost every game system is data-driven. Blocks, items, entities, recipes, biomes, structures, world generation, and more can all be extended without modifying the engine.

《世纪之旅(Century Journey)》是一款现代体素沙盒游戏，主打高性能、高可模组性与长期可拓展性。
区别于传统沙盒游戏，本项目整套游戏体系均采用数据驱动设计。方块、物品、实体、合成配方、生物群系、建筑结构、世界生成等内容均可直接拓展，无需改动引擎底层源码。


---

# ✨ Features 特性
| English                                    | 中文                  |
|--------------------------------------------|---------------------|
| ⚡ Built with Rust                          | ⚡ 基于 Rust 语言开发      |
| 🎮 Powered by Bevy Engine                  | 🎮 Bevy 游戏引擎驱动      |
| 🌍 Infinite voxel world                    | 🌍 超大无限体素世界         |
| 📦 Data-driven content system              | 📦 完整数据驱动内容架构       |
| 🔧 Powerful Mod API                        | 🔧 完善的模组开发API       |
| 🎨 Full Resource Pack support              | 🎨 完整资源包支持          |
| 🌊 Dynamic world simulation (planned)      | 🌊 动态世界模拟（开发规划中）    |
| ☀️ Complete Day/Night cycle                | ☀️ 完整昼夜循环系统         |
| 🌧 Dynamic weather system (planned)        | 🌧 动态天气系统（开发规划中）    |
| 🌱 Ecology & seasonal mechanisms (planned) | 🌱 生态系统与四季机制（开发规划中） |
| ⚒ Cross-platform Multiplayer (planned)     | ⚒ 跨平台多人联机（开发规划中）    |
| 🖥 Native Cross-platform support           | 🖥 原生全平台适配          |

---

# 🏗 Architecture

The project adopts a standard layered architecture for easy maintenance and expansion.
本项目采用标准分层架构，便于维护与二次拓展。

```
Engine
│
├── Rendering
├── Asset System
├── ECS Extensions
├── Resource Runtime
└── Core Infrastructure

Shared
│
├── Identifier
├── Runtime Types
├── Registry Traits
└── Utilities

Content
│
├── Blocks
├── Items
├── Biomes
├── Recipes
├── Loot Tables
├── Entities
└── Data Loaders

Game
│
├── World
├── Player
├── Inventory
├── Physics
├── Crafting
└── Gameplay

Client
│
├── Rendering
├── UI
├── Audio
├── Input
└── Animation

Server
│
├── Networking
├── Synchronization
├── Saving
└── Multiplayer
```

---

# 📦 Modding Support 模组开发

Century Journey is designed from the beginning to fully support modding.
《世纪之旅》在立项之初便将模组生态纳入核心设计，原生支持模组开发。
The project is currently under active development.
项目目前处于积极开发阶段。

---

# 🚀 Building 构建

Ensure you have a full Rust environment (with Cargo) installed locally.
Pull the project repository locally, run cargo build for compilation, 
and use cargo run to start the application.

配置好完整的本地 Rust 运行环境（包含 Cargo 工具），
拉取项目代码至本地，执行 cargo build 命令完成项目构建，
执行 cargo run 命令即可运行程序。

---

# 🤝 Contributing

Contributions are warmly welcome!
欢迎各位开发者提交代码与内容贡献！
Before submitting a Pull Request, please check the documents below:
提交 PR 前，请务必阅读以下规范文件（待完善）:
CONTRIBUTING.md（开发贡献规范・待编写）
CODE_OF_CONDUCT.md（行为准则・待编写）
CLA.md（贡献者协议・待编写）


---

# 📚 Documentation

All official documents will be placed in the repository.
官方全套开发文档后续将统一放置于仓库内。
TODO

---

# 🗺 Development Roadmap 开发路线图
- [x] Chunk System 区块系统
- [x] Block System 方块系统
- [x] Inventory    背包系统(待优化)
- [x] ViewModel    手持物品渲染(待优化)
- [x] Item System  物品系统
- [ ] Entity System 实体系统
- [ ] World Generation 2.0   二代世界生成器
- [ ] Mod SDK     官方模组开发套件
- [ ] Multiplayer  多人联机
- [ ] Dedicated Server   独立专用服务器
- [ ] Steam Release      平台正式上线

---

# 📄 License 许可证

Century Journey uses **multiple licenses** depending on the project component.
《世纪之旅》根据项目组成部分使用**多个许可证**。

| Component            | License                                        |
|----------------------|------------------------------------------------|
| Source Code 项目源码     | **Mozilla Public License 2.0 (MPL-2.0)**       |
| Game Assets 游戏资源     | **Century Journey Assets License (CJAL v1.0)** |
| Mod SDK/API 模组SDK接口	 | **MIT License**                                |
| Documentation 文档     | **CC BY 4.0**                                  |

## Source Code 项目源码

The source code located in the `src/` directory is licensed under the **Mozilla Public License 2.0 (MPL-2.0)**.

存放于 src/ 目录下的源代码，遵循 Mozilla 公共许可证 2.0 版（MPL-2.0） 协议。

## Game Assets 游戏资源

All textures, models, sounds, music, icons, animations, shaders, and other game assets are licensed under the **Century Journey Assets License (CJAL v1.0)**.
Unless explicitly permitted by the asset license, official assets may **not** be used in other games or commercial products.

所有贴图、模型、音效、配乐、图标、动画、着色器及其余游戏资源，均遵循 ** 世纪之旅资源许可证（CJAL v1.0）** 协议。

## Mod SDK  模组开发

The official Mod SDK/API is licensed under the **MIT License** to encourage community development.

为助力社区开发，官方模组 SDK/API 采用 MIT 许可证 开源。

## Documentation 文档

Project documentation is licensed under **Creative Commons Attribution 4.0 International (CC BY 4.0)**.

本项目所有文档遵循 知识共享署名 4.0 国际许可协议（CC BY 4.0）。

---

# ❤️ Special Thanks

Thanks to:
致谢名单：

- Rust Community
  Rust 官方社区
- Bevy Community
  Bevy 引擎社区
- Every contributor
  所有项目贡献者
- Every player
  每一位游玩的玩家

---

<div align="center">

**Made with ❤️ using Rust**

Century Journey © 2026 Contributors
《世纪之旅》 © 2026 全体项目贡献者
</div>
