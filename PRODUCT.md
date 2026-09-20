# Product

## Register

product

## Platform

Tauri desktop application for macOS, Windows, and Linux on x64 and arm64.

## Users

DeepSeek Harness users, developers, and plugin authors who need to test fast-moving DSH versions without contaminating their system Node.js environment or replacing a known-good DSH installation.

## Product purpose

`dsh-launcher` turns an unstable command-line environment into a compact, visible, and reversible desktop launch flow. Users explicitly choose a Node version, a DSH version, and a workspace before launching. The app owns the private runtime, reports real process state, opens the Web UI, and can stop or safely restart the exact process it started.

## Product promises

1. System Node, npm, and global `dsh` are not required.
2. DSH versions coexist and are changed only through explicit user actions.
3. Node 24 is the recommended default; Node 26 is downloaded only on demand.
4. Runtime and workspace state remains visible before launch.
5. Online version checks and installs preserve existing working versions.
6. Chinese and English interfaces follow the user's system or saved preference.

## Positioning

A small desktop control surface for isolated DSH runtimes—not a terminal emulator, package-manager dashboard, or OS security sandbox.

## Brand personality

Compact, calm, technical, and trustworthy. The interface borrows the restrained dark visual language of DeepSeek Harness while keeping actions and runtime state explicit.

## Safety boundaries

- The WebView exposes only bounded Tauri commands, not a generic shell.
- Runtime isolation covers files, versions, and environment variables.
- DSH processes still run with the current user's operating-system permissions.
- Destructive version removal is unavailable for the active or default version.
