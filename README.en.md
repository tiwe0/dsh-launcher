<p align="center">
  <img src="docs/banner.svg" alt="dsh launcher banner" />
</p>

<p align="center">
  <a href="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml"><img src="https://github.com/tiwe0/dsh-launcher/actions/workflows/build.yml/badge.svg" alt="Build status" /></a>
  <a href="https://github.com/tiwe0/dsh-launcher/releases"><img src="https://img.shields.io/github/v/release/tiwe0/dsh-launcher?display_name=tag" alt="Latest release" /></a>
  <img src="https://img.shields.io/badge/Tauri-2-24C8DB?logo=tauri&logoColor=white" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/Node-24%20%7C%2026-5FA04E?logo=nodedotjs&logoColor=white" alt="Node 24 and 26" />
</p>

<p align="center">
  <a href="README.md">中文</a> · <strong>English</strong>
</p>

`dsh launcher` is a compact desktop launcher for DeepSeek Harness. It keeps Node, npm, and DSH in an isolated runtime, supports side-by-side DSH versions, and turns version selection, workspace selection, launch, stop, and restart into one small desktop screen.

## Interface preview

<p align="center">
  <img src="docs/launcher.png" alt="dsh launcher launch screen" />
</p>

## Why dsh launcher

DeepSeek Harness is evolving quickly, and releases may introduce breaking changes. Updating a global installation can disrupt an existing workflow while mixing Node, npm, and DSH state with the host environment.

`dsh launcher` keeps those changes inside its own application data. You can evaluate a new DSH version without replacing the version that already works or modifying the system Node installation.

## Product capabilities

- **Isolated runtime**: no dependency on system Node, npm, or a global `dsh` installation.
- **Side-by-side DSH versions**: install, remove, switch, and select a default `@deepseek-ai/dsh` version.
- **Online version management**: discover remote DSH releases, then download and install them inside the app.
- **Controlled Node versions**: Node 24 is the recommended default; Node 26 is downloaded only when selected.
- **One-click runtime control**: automatically open the DSH Web UI after launch, then stop or safely restart it.
- **Isolated workspace default**: use `~/.dsh-launcher` and create it automatically when missing.
- **China-friendly mirror defaults**: npmmirror is configured for Node downloads and the npm registry.
- **Aligned CLI environment**: generate a private `dsh` command and place it plus the private Node runtime first in child-process `PATH`, keeping plugin operations on the launcher's profile.
- **Compact desktop experience**: custom title bar, DeepSeek Harness visual language, and smooth state transitions.
- **Chinese and English UI**: detect the system language, persist the choice, and toggle instantly.

## Supported platforms

| Platform | x64 | arm64 |
| --- | :---: | :---: |
| macOS | CI build | CI build |
| Windows | CI build | CI build |
| Linux | CI build | CI build |

GitHub Actions builds all six combinations on native runners. Pushing a `v*` tag collects every platform bundle and automatically publishes a GitHub Release.

## Quick start

### Download an application bundle

1. Open [Releases](https://github.com/tiwe0/dsh-launcher/releases).
2. Download the archive matching your operating system and CPU architecture.
3. Extract and launch `dsh launcher`.
4. Keep the recommended Node 24, choose a DSH version and workspace, then select Launch.

Current bundles are not Apple-notarized, Windows code-signed, or signed for a Linux package repository. Your operating system may display an unsigned application warning.

### Local development

Install Node 24, Rust stable, and the [Tauri 2 platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```bash
npm ci
npm run build
npm run tauri dev
```

Run the complete local validation set:

```bash
npm run prepare:sidecar
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/sidecar/Cargo.toml
```

## Isolation model

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

- macOS and Linux use bundled NVM resources to manage the private Node runtime.
- Windows uses a private portable Node runtime without relying on WSL, Git Bash, or system Node.
- DSH receives its own `DSH_HOME`, with every version installed in a separate directory.
- DSH inherits the current user's `HOME`, so Git, SSH, and shell configuration remain available. Set `DSH_LAUNCHER_ISOLATE_HOME=1` to opt into a fully isolated HOME.
- The app displays the private `DSH_HOME` and CLI shim so the same profile can be used from a terminal.
- Runtime state and version metadata remain in the user's application data directory.

This is **runtime and version isolation**, not an operating-system security sandbox. DSH child processes still run with the permissions of the current user.

## Current boundaries

- DSH may still introduce breaking changes. Side-by-side versions reduce upgrade risk but cannot guarantee third-party compatibility.
- CI artifacts are currently unsigned and are intended for development, testing, and internal validation.
- The launcher currently exposes only Node 24 and 26, and downloads Node 24 by default.
- China-friendly mirrors are the current defaults and do not yet have a graphical configuration screen.

## Technology

- Tauri 2 + Rust
- React 19 + TypeScript + Vite
- Material UI + motion transitions
- i18next + react-i18next
- A native Rust sidecar for Node/DSH installation, version management, and process control

> `dsh launcher` is a community project and is not an official DeepSeek distribution. DeepSeek, DeepSeek Harness, and their related marks belong to their respective owners.

## Star History

<a href="https://www.star-history.com/?repos=tiwe0%2Fdsh-launcher&type=date&legend=top-left">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&theme=dark&legend=top-left" />
    <source media="(prefers-color-scheme: light)" srcset="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
    <img alt="Star History Chart" src="https://api.star-history.com/chart?repos=tiwe0/dsh-launcher&type=date&legend=top-left" />
  </picture>
</a>
