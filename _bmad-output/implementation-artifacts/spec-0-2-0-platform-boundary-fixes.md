---
title: '0.2.0 platform boundary fixes'
type: 'bugfix'
created: '2026-08-03'
status: 'in-review'
baseline_commit: '25c3f7d32a852cc018c501e2cbe214cd57907470'
context:
  - 'docs/architecture.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `0.2.0` 的 Windows Release 在 Clippy 阶段因 macOS 专用代码未完整条件编译而失败；快速入门同时向 Windows 展示了并不存在的 Space 原生预览操作。Windows 焦点结果和托盘主题刷新也存在已确认的平台边界缺口。

**Approach:** 保留统一的跨平台业务接口，将实现和 UI 能力按目标平台收紧：macOS 独占 Quick Look/Accessibility，Windows 隐藏快速入门预览入口并加强 Win32 焦点与托盘行为。

## Boundaries & Constraints

**Always:** 保持快速入门的图片、文本、链接及 Enter 模拟复制在双端可用；保持真实历史记录在 Windows 的系统打开回退；所有平台专用代码必须能通过各自的 `cargo clippy -D warnings`。

**Ask First:** 需要新增 Windows 应用内预览、改变自动粘贴权限模型、删除 Windows 前台激活兼容回退，或改变发布版本号。

**Never:** 不在 Windows 模拟 Quick Look；不通过 `allow` 关闭 lint；不让 Windows 调用 macOS 系统设置或 Accessibility API；不宣称未执行的 Windows 真机测试已经通过。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|---------------------------|----------------|
| Windows 快速入门 | Practice 页面 | 不展示 Space 预览，也不处理 Space | 选择与 Enter 演示保持可用 |
| macOS 快速入门 | Practice 页面 | 继续展示并支持 Space Quick Look | 维持现有错误提示 |
| Windows 面板激活 | 窗口成为 foreground 但键盘焦点设置失败 | 不报告已获得焦点 | 保持生命周期等待真实焦点事件 |
| Windows 主题变化 | 应用收到系统主题变化事件 | 重新读取任务栏主题并更新托盘图标 | 失败只记录日志，不中断窗口事件 |

</frozen-after-approval>

## Code Map

- `src/lib/onboarding/OnboardingView.svelte` -- 平台相关操作提示与键盘入口。
- `src-tauri/src/commands/onboarding.rs` -- macOS 系统设置命令的条件编译边界。
- `src-tauri/src/preview/mod.rs` -- macOS Quick Look 示例资源及参数使用。
- `src-tauri/src/window/windows.rs` -- Windows 前台与键盘焦点结果。
- `src-tauri/src/tray.rs` -- Windows 托盘图标选择和刷新。
- `src-tauri/src/lib.rs` -- 系统主题窗口事件接入。

## Tasks & Acceptance

**Execution:**
- [x] `src/lib/onboarding/OnboardingView.svelte` -- 仅在 macOS 展示和处理 Space 预览。
- [x] `src-tauri/src/commands/onboarding.rs`, `src-tauri/src/preview/mod.rs` -- 对 macOS 专用导入、资源、函数和参数施加一致的条件编译。
- [x] `src-tauri/src/window/windows.rs` -- 将 Win32 键盘焦点设置结果纳入激活结果。
- [x] `src-tauri/src/tray.rs`, `src-tauri/src/lib.rs` -- Windows 系统主题变化时安全刷新托盘图标。
- [x] 相关测试 -- 锁定平台 UI、焦点结果辅助逻辑和托盘图标选择行为。

**Acceptance Criteria:**
- Given Windows 构建，when 执行 Release 中的 Rust lint，then 不再出现 macOS Quick Look 或 Accessibility 相关未使用错误。
- Given Windows 快速入门，when 用户查看操作说明并按 Space，then 不承诺或触发不可用的预览。
- Given macOS 快速入门，when 用户按 Space，then 现有 Quick Look 行为保持不变。
- Given Windows 主题或焦点 API 返回异常，when ClipClop 响应事件，then 状态不会被错误报告且应用继续运行。

## Spec Change Log

- 2026-08-03 review: Windows focus verification now accepts the top-level HWND or its WebView child; non-macOS Space is consumed without preview; taskbar theme refresh now watches `SystemUsesLightTheme` registry changes instead of relying on the independently configurable app theme. KEEP: macOS Quick Look behavior and the existing Windows foreground-lock fallback.

## Verification

**Commands:**
- `pnpm check && pnpm test && pnpm build` -- 2026-08-03：0 个诊断、41 个测试与生产构建通过。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- 2026-08-03：通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` -- 2026-08-03：macOS 当前目标通过。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-gnu --all-targets -- -D warnings` -- 2026-08-03：Windows 条件编译与 lint 通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- 2026-08-03：47 个测试通过。

**Manual checks:**
- [ ] Windows 真机确认快速入门无 Space 预览提示、召回后键盘可用、系统主题变化后托盘可见。
- [ ] macOS 真机确认快速入门 Space Quick Look 未回归。

## Suggested Review Order

**平台能力入口**

- Windows 隐藏并消费不可用的 Space 预览，macOS 行为保持不变。
  [`OnboardingView.svelte:96`](../../src/lib/onboarding/OnboardingView.svelte#L96)

- 单一能力判断同时驱动提示和键盘行为。
  [`api.ts:29`](../../src/lib/onboarding/api.ts#L29)

**原生平台边界**

- Windows 激活只有在顶层窗口或 WebView 子窗口获得焦点时才成功。
  [`windows.rs:78`](../../src-tauri/src/window/windows.rs#L78)

- 注册表监听精确跟随 Windows 任务栏主题并在主线程刷新托盘。
  [`tray.rs:155`](../../src-tauri/src/tray.rs#L155)

- macOS 快速入门资源只进入 macOS 与测试构建。
  [`mod.rs:20`](../../src-tauri/src/preview/mod.rs#L20)

- macOS Accessibility 错误类型不再污染 Windows 编译。
  [`onboarding.rs:10`](../../src-tauri/src/commands/onboarding.rs#L10)

**回归证据**

- 平台能力测试锁定 macOS 支持、Windows 不支持。
  [`api.test.ts:8`](../../src/lib/onboarding/api.test.ts#L8)

- 激活结果测试拒绝缺少键盘焦点的成功状态。
  [`windows.rs:135`](../../src-tauri/src/window/windows.rs#L135)
