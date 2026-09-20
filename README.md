<p align="center">
  <img src="docs/banner.svg" alt="dsh launcher banner" />
</p>

<p align="center">
  <a href="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml"><img src="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml/badge.svg" alt="构建状态" /></a>
  <a href="https://github.com/tiwe0/dsh-launcher/releases"><img src="https://img.shields.io/github/v/release/tiwe0/dsh-launcher?display_name=tag" alt="最新版本" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Node-24%20%7C%2026-5FA04E?logo=nodedotjs&logoColor=white" alt="Node 24 与 26" />
</p>

<p align="center">
  <strong>中文</strong> · <a href="README.en.md">English</a>
</p>

`dsh launcher` 是面向 DeepSeek Harness 的轻量级桌面启动器。应用将 Node.js、npm 与 DSH 纳入独立运行环境，支持多个 DSH 版本并行安装，并通过统一界面提供版本选择、工作目录配置以及启动、停止和重启等运行控制。

## 界面预览

<p align="center">
  <img src="docs/launcher.png" alt="dsh launcher 启动页面" />
</p>

## 为什么需要它

DeepSeek Harness（DSH）仍处于快速迭代阶段，新版本可能引入不兼容变更。直接更新全局安装可能影响既有工作流，并使 Node.js、npm 与 DSH 的运行状态与系统环境相互混杂。

`dsh launcher` 将相关组件与版本数据保存在应用专用目录中，使用户能够在不替换已验证版本、也不修改系统 Node.js 环境的前提下评估新版本。

## 产品能力

- **轻量原生桌面应用**：采用 Tauri 与系统 WebView，不额外打包浏览器内核，且不捆绑与 DSH 启动无关的插件或扩展功能。
- **私有运行时**：由应用统一管理 Node.js、npm 与 DSH，无需依赖系统级安装；默认推荐 Node.js 24，并仅在需要时下载 Node.js 26。
- **DSH 版本管理**：支持在线检索与安装 DSH 版本，以及多版本并存、切换、删除和默认版本设置；安装新版本不会覆盖现有可用版本。
- **完整运行控制**：在指定工作目录内启动 DSH Web，并对应用创建的进程执行停止或安全重启；默认工作目录 `~/.dsh-launcher` 不存在时将自动创建。
- **一致的命令环境**：将私有 `dsh` 与 Node.js 可执行文件置于子进程 `PATH` 前端，并默认通过 npmmirror 获取 Node.js 与 npm 包，避免插件安装至其他运行配置。

## 快速开始

### 下载应用

1. 访问 [Releases](https://github.com/tiwe0/dsh-launcher/releases) 页面。
2. 下载与当前操作系统及 CPU 架构相匹配的压缩包。
3. 解压后启动 `dsh launcher`。
4. 使用默认推荐的 Node.js 24，选择 DSH 版本与工作目录，然后单击“启动”。

当前发布产物尚未经过 Apple 公证、Windows 代码签名或 Linux 软件源签名，操作系统可能因此显示未签名应用警告。

### 本地开发

本地开发环境需要 Node.js 24、Rust stable，以及 [Tauri 2 对应平台的系统依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm ci
npm run build
npm run tauri dev
```

执行以下命令以完成完整验证：

```bash
npm run prepare:sidecar
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/sidecar/Cargo.toml
```

## 隔离方式

```text
React + MUI
    -> Tauri commands
    -> dsh-runtime native sidecar
    -> private Node runtime
       - Node 24.21.0: recommended and installed first
       - Node 26.9.0: downloaded on demand
    -> versioned DSH installations
    -> selected workspace
```

- macOS 与 Linux 通过随应用分发的 NVM 资源管理私有 Node.js 运行时。
- Windows 使用应用专用的便携式 Node.js 运行时，无需依赖 WSL、Git Bash 或系统 Node.js。
- DSH 使用独立的 `DSH_HOME`，各版本分别安装至独立目录。
- 默认情况下，DSH 继承当前用户的 `HOME`，以继续使用现有 Git、SSH 与 shell 配置；如需隔离 `HOME`，可设置 `DSH_LAUNCHER_ISOLATE_HOME=1`。
- 应用界面显示私有 `DSH_HOME` 与命令行 shim，可复制相应命令，以便从终端访问同一运行配置。
- 运行状态与版本元数据存储于用户应用数据目录。

此处的隔离范围仅包括**运行环境与版本数据**，不构成操作系统权限沙箱。DSH 子进程仍以当前用户权限运行。

## 当前限制

- DSH 后续版本仍可能引入不兼容变更；多版本并存能够降低升级风险，但无法保证第三方组件的兼容性。
- CI 产物目前尚未签名，仅建议用于开发、测试与内部验证。
- 当前仅支持 Node.js 24 与 26，默认仅下载 Node.js 24。
- npmmirror 目前为默认镜像源，尚未提供图形化配置入口。

## 技术栈

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Material UI + Motion 动画
- i18next + react-i18next
- 原生 Rust sidecar 负责 Node.js 与 DSH 的安装、版本管理及进程控制

> `dsh launcher` 是社区项目，并非 DeepSeek 官方发行的软件。DeepSeek、DeepSeek Harness 及相关标识归其各自权利人所有。

## Star History

<a href="https://www.star-history.com/?repos=tiwe0%2Fdsh-launcher&type=date&legend=top-left">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&theme=dark&legend=top-left" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
    <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
  </picture>
</a>
