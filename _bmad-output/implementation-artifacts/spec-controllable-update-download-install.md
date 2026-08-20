---
title: 'Controllable update download and install'
type: 'feature'
created: '2026-08-20'
status: 'done'
baseline_commit: 'b1bf64d342cc961b5a31be8d1e553915fdac8fbd'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/docs/interaction-contract.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前更新把下载、安装、重启合成不可中断动作；用户不能只下载、稍后安装或真正停止网络下载。手动检查还会清空左侧已确认的“已是最新版本”。

**Approach:** 拆成可取消下载与明确安装两阶段；“下载并安装”复用下载后自动安装。检查期间保留最后确认状态，仅由按钮和 live region 表达检查中。

## Boundaries & Constraints

**Always:** 取消必须终止请求并丢弃部分字节；仅保存通过现有签名验证的包；模块级 store 保持关闭/重开设置后的真实状态；下载完成后显示“更新已下载，等待安装”，退出应用或发现更高版本时自然清理；只有“下载并安装”或“安装并重启”可触发安装；遵守 Windows 安装时退出进程的限制；使用原生按钮和现有 CSS token；开发环境可通过 URL 参数预览更新状态且不进入生产逻辑。

**Ask First:** 新增依赖、落盘保存安装包、改变 manifest 或自动检查语义。

**Never:** 不提供含义模糊的“放弃更新”按钮；不跨进程保留下载；不自动恢复取消任务；不伪装取消；不增加队列、多版本缓存或下载管理器。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 仅下载 | 有可用版本 | 显示进度；验证后提供“安装并重启” | 网络/签名失败可重试 |
| 下载并安装 | 有可用版本 | 下载成功后直接安装 | 下载失败不安装 |
| 取消下载 | 下载中 | 中止请求、清理字节、回到可下载 | 取消/完成竞态只有一个终态 |
| 稍后安装 | 已下载、应用未退出 | 关闭设置后继续保留；重开后可无重复下载地安装 | 退出应用或发现更高版本后要求重下 |
| 检查期间 | 已确认最新版本 X | 左侧保持；按钮忙碌，live region 宣布 | 结果或错误返回后替换 |
| 开发预览 | `?updatePreview=<state>` | 用固定版本和进度渲染指定状态 | 未知值忽略 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/updater.rs` -- 受控任务、取消句柄、已验证包和安装命令。
- `src-tauri/src/commands/mod.rs`、`src-tauri/src/lib.rs` -- 注册状态与命令。
- `src/lib/updater/api.ts` -- IPC 封装，保留检查/缓存；更高版本使旧下载失效。
- `src/lib/updater/store.svelte.ts` -- 下载、已下载、安装、取消与错误状态。
- `src/lib/settings/SettingsView.svelte`、`src/lib/i18n/catalogs.ts` -- 动作、进度、文案和检查状态。
- updater tests -- 取消竞态与状态清理。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/updater.rs`、注册点 -- 实现单任务下载、真实取消、内存保存和安装。
- [x] `src/lib/updater/api.ts`、`store.svelte.ts` -- 组合 download/cancel/install，保留当前进程内的完成状态并处理竞态。
- [x] `SettingsView.svelte`、catalogs -- 提供全部动作与反馈，检查时保留左侧状态。
- [x] updater tests -- 覆盖取消、仅下载、自动安装与状态保持。
- [x] `store.svelte.ts` -- 提供仅 DEV 生效的 URL 状态预览并验证参数。

**Acceptance Criteria:**
- Given 新版本可用，when 选择任一模式，then 只启动一次下载并按模式停留或安装。
- Given 下载中，when 取消，then 传输停止且可重新下载。
- Given 已下载，when 关闭再打开设置，then 仍可安装；退出应用后不保留。
- Given 已显示最新版本，when 手动检查，then 左侧保持且按钮独立忙碌。

## Spec Change Log

- 2026-08-20：按用户要求增加开发态更新样式预览；不改变生产更新流程。
- 2026-08-20：复核后修复取消/完成、版本变更、安装/重启竞态，并统一设置按钮规范。

## Design Notes

Tauri 2.10.1 可分开下载/安装，但 JS 没有 AbortSignal。真正取消需由 Rust 持有下载 future 的取消信号；约 15 MB 的包只在当前进程内保存即可。

## Verification

**Commands:**
- `pnpm test && pnpm check` -- 前端测试与诊断通过。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- 格式正确。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` -- 无警告。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 测试通过。

**Manual checks (if no CLI):**
- macOS/Windows 验证取消、重开设置、手动安装和退出/重启。

## Suggested Review Order

**更新状态入口**

- 单一 store 协调检查、下载、取消、安装和重启终态。
  [`store.svelte.ts:89`](../../src/lib/updater/store.svelte.ts#L89)

- 设置页按状态呈现动作，同时保留已确认检查结果。
  [`SettingsView.svelte:447`](../../src/lib/settings/SettingsView.svelte#L447)

**原生下载边界**

- 后端持有可中止任务，并对竞态采用单一终态。
  [`updater.rs:92`](../../src-tauri/src/commands/updater.rs#L92)

- 安装复制已验证包后释放锁，失败仍可重试。
  [`updater.rs:254`](../../src-tauri/src/commands/updater.rs#L254)

- IPC 层按 requestId 隔离事件并拆分重启失败。
  [`api.ts:233`](../../src/lib/updater/api.ts#L233)

**设计与验证**

- 更新动作遵循统一 ghost 按钮尺寸与现有 token。
  [`SettingsView.svelte:533`](../../src/lib/settings/SettingsView.svelte#L533)

- 状态测试覆盖仅下载、取消和仅重试重启。
  [`store.test.ts:29`](../../src/lib/updater/store.test.ts#L29)
