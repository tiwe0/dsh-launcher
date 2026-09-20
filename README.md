<p align="center">
  <img src="public/deepseek-mark.svg" width="88" alt="dsh-launcher logo" />
</p>

<h1 align="center">dsh-launcher</h1>

<p align="center">
  A compact, isolated desktop launcher and version manager for DeepSeek Harness.
</p>

<p align="center">
  <a href="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml"><img src="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml/badge.svg" alt="Build status" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Node-24%20%7C%2026-5FA04E?logo=nodedotjs&logoColor=white" alt="Node 24 and 26" />
</p>

<p align="center">
  <strong>中文</strong> · <a href="#english">English</a>
</p>

## 为什么需要 dsh-launcher

DeepSeek Harness（DSH）仍在快速迭代，不同版本可能包含破坏性变化。直接更新全局安装容易让现有工作流失效，也会把 Node、npm 和 DSH 的状态混入系统环境。

`dsh-launcher` 将运行环境收进一个小巧的桌面启动器：选择 Node 与 DSH 版本、选择工作目录，然后启动。不同版本独立安装、显式切换，可以验证新版本而不覆盖当前可用版本。

## 产品能力

- **独立运行时**：不依赖系统 Node、npm 或全局 `dsh`。
- **DSH 版本隔离**：多个 `@deepseek-ai/dsh` 版本并存，可安装、删除并设置默认版本。
- **在线版本管理**：检测远端版本并在应用内下载安装。
- **Node 版本控制**：默认准备推荐的 Node 24；Node 26 仅在用户选择后按需下载。
- **一键控制**：启动后自动打开 DSH Web；支持关闭和安全重启。
- **工作目录隔离**：默认使用 `~/.dsh-launcher`，目录不存在时自动创建。
- **国内镜像默认值**：Node 使用 npmmirror 镜像，npm 使用 npmmirror registry。
- **紧凑桌面体验**：无系统标题栏、DeepSeek Harness 视觉风格、流畅状态动画。
- **中英文界面**：自动检测系统语言、记住用户选择，并支持即时切换。

## 支持平台

| 平台 | x64 | arm64 |
| --- | :---: | :---: |
| macOS | CI 构建 | CI 构建 |
| Windows | CI 构建 | CI 构建 |
| Linux | CI 构建 | CI 构建 |

GitHub Actions 在对应架构的原生 runner 上构建六个平台组合。每次构建的安装包或应用包可在对应 workflow run 的 Artifacts 中下载。

## 快速开始

### 使用构建产物

1. 打开仓库的 [Actions](https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml)。
2. 选择最近一次成功的 `Build desktop matrix` 运行。
3. 下载与操作系统和 CPU 架构匹配的 artifact。
4. 启动应用，保留推荐的 Node 24 或选择 Node 26，再选择 DSH 版本与工作目录。

当前构建未进行 Apple notarization、Windows code signing 或 Linux 仓库签名，操作系统可能显示未签名应用提示。

### 本地开发

需要 Node 24、Rust stable，以及 [Tauri 2 对应平台的系统依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm ci
npm run build
npm run tauri dev
```

运行完整验证：

```bash
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

- macOS 与 Linux 使用打包的 NVM 资源管理私有 Node。
- Windows 使用应用私有的便携 Node，不依赖 WSL、Git Bash 或系统 Node。
- DSH 使用独立的 `DSH_HOME`，每个版本安装在独立目录。
- 运行目录和版本元数据保存在用户应用数据目录中。

这是**运行环境与版本隔离**，不是操作系统权限沙箱。DSH 子进程仍拥有启动它的当前用户权限。

## 当前边界

- DSH 自身仍可能发生破坏性更新；启动器通过版本并存降低风险，但不能保证第三方版本兼容性。
- CI 产物目前未签名，适合开发测试和内部验证。
- Node 版本暂时只提供 24 和 26；默认仅下载 Node 24。
- 国内镜像是默认配置，网络环境需要时可在后续版本中扩展为可配置项。

## 技术栈

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Material UI
- i18next + react-i18next
- 原生 Rust sidecar 负责 Node/DSH 安装、版本管理与进程控制

> `dsh-launcher` 是社区项目，并非 DeepSeek 官方发行的软件。DeepSeek、DeepSeek Harness 及相关标识归其各自权利人所有。

---

<a id="english"></a>

## English

### What it does

`dsh-launcher` is a small desktop launcher that keeps DeepSeek Harness away from your global Node.js environment. It provides an isolated runtime, side-by-side DSH versions, online version detection and installation, explicit workspace selection, and one-click launch, stop, or restart controls.

### Product highlights

- Private Node and npm environment; no global `dsh` dependency.
- Node 24 by default, with Node 26 downloaded only when selected.
- Side-by-side DSH installations with an explicit default version.
- Online DSH version discovery, download, install, removal, and switching.
- Automatic Web UI opening after launch.
- Default workspace at `~/.dsh-launcher`, created automatically.
- Chinese mirror defaults for Node and npm.
- Chinese and English UI with system detection and persisted preference.
- Native CI builds for macOS, Windows, and Linux on x64 and arm64.

### Development

Install Node 24, Rust stable, and the platform prerequisites for Tauri 2, then run:

```bash
npm ci
npm run build
npm run tauri dev
```

The isolation boundary covers runtime files, versions, and environment variables. It is not an OS security sandbox; DSH processes still run with the current user's permissions.

> `dsh-launcher` is a community project and is not an official DeepSeek distribution.
