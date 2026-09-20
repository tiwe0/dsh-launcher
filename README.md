<p align="center">
  <img src="docs/banner.svg" alt="dsh-launcher banner" />
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

`dsh-launcher` 是一个小巧的 DeepSeek Harness 桌面启动器：它把 Node、npm 和 DSH 收进独立运行环境，让不同 DSH 版本并存，并用一个启动页面完成版本选择、工作目录选择、启动、关闭与重启。

## 为什么需要它

DeepSeek Harness（DSH）仍在快速迭代，不同版本可能包含破坏性变化。直接更新全局安装容易让现有工作流失效，也会把 Node、npm 和 DSH 的状态混入系统环境。

`dsh-launcher` 将这些变化隔离在应用自己的目录内。你可以验证新版本而不覆盖当前可用版本，也不需要修改系统 Node 环境。

## 产品能力

- **独立运行时**：不依赖系统 Node、npm 或全局 `dsh`。
- **DSH 版本隔离**：多个 `@deepseek-ai/dsh` 版本并存，可安装、删除、切换并设置默认版本。
- **在线版本管理**：检测远端 DSH 版本，在应用内完成下载和安装。
- **Node 版本控制**：默认准备推荐的 Node 24；Node 26 仅在选择后按需下载。
- **一键运行控制**：启动成功后自动打开 DSH Web，运行中可关闭或安全重启。
- **工作目录隔离**：默认使用 `~/.dsh-launcher`，目录不存在时自动创建。
- **国内镜像默认值**：Node 使用 npmmirror 镜像，npm 使用 npmmirror registry。
- **紧凑桌面体验**：无系统标题栏、DeepSeek Harness 视觉风格与流畅状态动画。
- **中英文界面**：自动检测系统语言、记住用户选择，并支持即时切换。

## 支持平台

| 平台 | x64 | arm64 |
| --- | :---: | :---: |
| macOS | CI 构建 | CI 构建 |
| Windows | CI 构建 | CI 构建 |
| Linux | CI 构建 | CI 构建 |

GitHub Actions 在对应架构的原生 runner 上构建六个平台组合。推送 `v*` tag 后，工作流会汇总全部平台产物并自动创建 GitHub Release。

## 快速开始

### 下载应用

1. 打开 [Releases](https://github.com/tiwe0/dsh-launcher/releases)。
2. 下载与操作系统和 CPU 架构匹配的压缩包。
3. 解压并启动 `dsh-launcher`。
4. 保留推荐的 Node 24，选择 DSH 版本和工作目录，然后点击启动。

当前构建尚未进行 Apple notarization、Windows code signing 或 Linux 仓库签名，操作系统可能显示未签名应用提示。

### 本地开发

需要 Node 24、Rust stable，以及 [Tauri 2 对应平台的系统依赖](https://v2.tauri.app/start/prerequisites/)。

```bash
npm ci
npm run build
npm run tauri dev
```

运行完整验证：

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

- macOS 与 Linux 使用打包的 NVM 资源管理私有 Node。
- Windows 使用应用私有的便携 Node，不依赖 WSL、Git Bash 或系统 Node。
- DSH 使用独立的 `DSH_HOME`，每个版本安装在独立目录。
- 运行目录和版本元数据保存在用户应用数据目录中。

这是**运行环境与版本隔离**，不是操作系统权限沙箱。DSH 子进程仍拥有启动它的当前用户权限。

## 当前边界

- DSH 自身仍可能发生破坏性更新；版本并存可以降低升级风险，但不能保证第三方版本兼容性。
- CI 产物目前未签名，适合开发测试和内部验证。
- Node 版本暂时只提供 24 和 26；默认仅下载 Node 24。
- 国内镜像为当前默认配置，尚未提供图形界面配置入口。

## 技术栈

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Material UI + Motion 动画
- i18next + react-i18next
- 原生 Rust sidecar 负责 Node/DSH 安装、版本管理与进程控制

> `dsh-launcher` 是社区项目，并非 DeepSeek 官方发行的软件。DeepSeek、DeepSeek Harness 及相关标识归其各自权利人所有。

## Star History

<a href="https://www.star-history.com/?repos=tiwe0%2Fdsh-launcher&type=date&legend=top-left">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&theme=dark&legend=top-left" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
    <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
  </picture>
</a>
