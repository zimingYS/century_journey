# Contributing to Century Journey
# 参与贡献《世纪之旅》

Thanks for your interest in Century Journey. This project is still in early active development, so focused issues and small pull requests are the easiest to review.

感谢你关注《世纪之旅》。本项目仍处于早期积极开发阶段，因此聚焦的问题反馈和范围较小的 Pull Request 最容易被审阅。

## Before You Start
## 开始之前

- Check existing issues and pull requests before opening a new one.
  提交新的 issue 或 Pull Request 前，请先检查已有内容。
- Open an issue first for large gameplay, architecture, networking, asset pipeline, or licensing changes.
  如果改动涉及大型玩法、架构、网络、资源管线或许可证，请先创建 issue 讨论。
- Keep pull requests scoped to one feature or fix.
  请让每个 Pull Request 聚焦于一个功能或一个修复。
- Do not include generated build output, local save files, IDE metadata, or unrelated formatting churn.
  请不要提交构建产物、本地存档、IDE 元数据或无关的格式化改动。

## Development Setup
## 开发环境

1. Install the latest stable Rust toolchain.
   安装最新稳定版 Rust 工具链。
2. Clone the repository.
   克隆本仓库。
3. Run `cargo fmt`.
   执行 `cargo fmt`。
4. Run `cargo test`.
   执行 `cargo test`。
5. Run `cargo build`.
   执行 `cargo build`。

## Pull Request Checklist
## Pull Request 检查清单

- The change builds locally.
  改动可以在本地成功构建。
- Tests pass, or the Pull Request explains why they cannot be run.
  测试能够通过；如果无法运行，请在 Pull Request 中说明原因。
- New behavior is covered by tests where practical.
  在可行的情况下，请为新增行为补充测试。
- Public assets, fonts, sounds, and other non-code files have clear license permission.
  公共资源、字体、音效和其他非代码文件需要有明确的许可证授权。
- User-facing changes update README or documentation when needed.
  面向用户的改动应在需要时同步更新 README 或相关文档。

## Licensing
## 许可证

Century Journey uses multiple licenses:

《世纪之旅》使用多种许可证：

- Source code: MPL-2.0.
  项目源码：MPL-2.0。
- Official game assets: Century Journey Assets License (CJAL v1.0).
  官方游戏资源：Century Journey Assets License（CJAL v1.0）。
- Mod SDK and API: MIT License.
  模组 SDK 与 API：MIT License。
- Documentation: CC BY 4.0.
  文档：CC BY 4.0。

By contributing, you agree that your contribution is provided under the license that applies to the part of the project you modify.

提交贡献即表示你同意：你的贡献将按照其修改部分对应的项目许可证提供。
