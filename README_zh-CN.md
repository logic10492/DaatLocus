<div align="center">

# Daat Locus

<img src="assets/logo.svg" alt="Logo" style="width:220px; height:auto;" />

[![English][readme-en-badge]][readme-en-url]
[![Crates.io][crates-badge]][crates-url]
[![CI][ci-badge]][ci-url]
[![License][license-badge]][license-url]

一个真正拥有经验的 agent runtime。

</div>

[readme-en-badge]: https://img.shields.io/badge/README-English-blue.svg?style=for-the-badge
[readme-en-url]: README.md
[crates-badge]: https://img.shields.io/crates/v/daat-locus?style=for-the-badge
[crates-url]: https://crates.io/crates/daat-locus
[ci-badge]: https://img.shields.io/github/actions/workflow/status/shadow3aaa/DaatLocus/ci.yml?style=for-the-badge&label=CI
[ci-url]: https://github.com/shadow3aaa/DaatLocus/actions/workflows/ci.yml
[license-badge]: https://img.shields.io/badge/License-Apache%202.0-blue.svg?style=for-the-badge
[license-url]: LICENSE

## 这是什么

Daat Locus 是一个长期运行在本地、由工具驱动的 agent runtime。

它适合那些会从历史中变得更好的工作：长期维护同一个项目、反复处理同一类任务、记住实践经验，并把经验沉淀成可复用的 runtime 结构。

Daat Locus 不是一次性聊天机器人封装。外部输入会作为结构化工作进入 runtime，模型负责语义判断，真正改变外部世界的动作必须通过显式工具发生。

## 快速开始

推荐使用 `cargo-binstall` 安装。它会下载与你的平台匹配的 GitHub Release 预编译二进制。正常安装不需要 Python、`uv` 或 PyInstaller。

```bash
cargo install cargo-binstall
cargo binstall daat-locus
```

也可以直接从 [GitHub Releases][releases-url] 下载对应平台的压缩包，解压后把 `daat-locus` 放进 `PATH`。

第一次启动时，Daat Locus 会进入交互式引导流程。

### 源码构建

`cargo install daat-locus` 会从 crates.io 源码编译。源码构建需要 Bun，因为 `build.rs` 会构建并嵌入 WebUI。

```bash
git clone https://github.com/shadow3aaa/DaatLocus
cd DaatLocus
cargo run --locked
```

`cargo build` 和 `cargo run` 会通过 `build.rs` 构建 WebUI，并默认把生成资源嵌入 daemon。需要本地 release 风格二进制时，直接运行 `cargo build -p daat-locus --release --locked`。

[releases-url]: https://github.com/shadow3aaa/DaatLocus/releases

## 常用入口

```bash
daat-locus run                 # 打开前台 runtime flow
daat-locus code <project-dir>  # 选择或创建项目作用域 session
daat-locus attach              # attach 到已有 daemon
daat-locus send "..."          # 发送一次消息并等待回复
daat-locus config              # 打开交互式配置菜单
```

## 文档

- [English README](README.md)
- [架构说明](docs/architecture_zh-CN.md)
- [配置](docs/configuration_zh-CN.md)
- [贡献指南](CONTRIBUTING_zh-CN.md)
- [内置 SOP primitives](workflows/README.md)

## 许可证

Daat Locus 使用 [Apache License 2.0](LICENSE)。
