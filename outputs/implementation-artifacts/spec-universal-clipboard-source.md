---
title: '识别并展示 Apple 通用剪贴板来源'
type: 'feature'
created: '2026-07-19'
status: 'done'
baseline_commit: '0af49652cbf85cf2070fd02c91873967fcc08373'
context:
  - '{project-root}/DESIGN.md'
  - '{project-root}/PRODUCT.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** macOS 从其他 Apple 设备同步来的剪贴板内容，目前会被尽力归因逻辑错误标记成 Mac 当前前台应用；界面也没有准确、易懂的远程来源表达。

**Approach:** 在 macOS 捕获开始时优先识别系统提供的远程剪贴板标记，把它归类为专用的通用剪贴板来源；界面显示单色双设备图标和“其他 Apple 设备”，不使用 AirDrop 或 Apple Logo，也不猜测具体设备和来源 App。

## Boundaries & Constraints

**Always:** 远程标记必须优先于声明来源、前台窗口和最近活动应用；用户可见名称固定为“其他 Apple 设备”；辅助提示说明“通过 Apple 通用剪贴板同步”；图标采用与现有灰阶设计一致的简洁双设备轮廓；原有应用来源及无来源记录保持兼容。

**Ask First:** 若验证发现当前剪贴板库无法读取远程标记，需要引入新的 macOS 原生依赖、数据库迁移或改变 `SourceApp` 公共结构时，先征求用户意见。

**Never:** 不显示 AirDrop 图标、Apple Logo、iPhone/iPad 等具体设备类型；不使用私有数据推断设备名称；不把这一来源加入“忽略此来源应用”的操作；不更改 Windows 的来源检测行为。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 通用剪贴板 | macOS 剪贴板格式含远程剪贴板标记 | 来源为“其他 Apple 设备”，显示双设备图标 | 不再回退到当前 Mac 应用 |
| 普通本机复制 | 不含远程标记且可识别来源应用 | 继续显示真实应用名称和图标 | 保持现状 |
| 无法识别来源 | 不含远程标记且所有归因方式均失败 | 只显示时间 | 不伪造来源 |
| 图标资源不可用 | 通用剪贴板来源无应用图标 | 使用内置双设备图标 | 不显示名称首字 fallback |

</frozen-after-approval>

## Code Map

- `src-tauri/src/clipboard/system.rs` -- 捕获剪贴板、读取可用格式、解析并冻结来源；需要在 macOS 来源链最前识别远程标记并补充单元测试。
- `src-tauri/src/clips/model.rs` -- 当前来源通过 `{ id, name }` 持久化；本次优先复用结构，避免数据库迁移。
- `src/routes/+page.svelte` -- 加载来源应用图标并渲染预览 metadata；需要识别专用来源 ID、跳过图标 IPC、绘制双设备图标及提示。
- `src/lib/clips/types.ts` -- 前端 `SourceApp` 类型，预计无需结构变更。
- `DESIGN.md` -- 来源归因与 metadata 视觉规范；记录通用剪贴板的命名、图标及交互边界。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/clipboard/system.rs` -- 定义稳定的通用剪贴板来源 ID/名称，优先检测 `com.apple.is-remote-clipboard`，并测试远程来源短路本机归因及普通来源不受影响。
- [x] `src/routes/+page.svelte` -- 为专用来源渲染可访问的单色双设备图标与说明，避免请求不存在的应用图标，并禁止对该来源执行“忽略来源应用”。
- [x] `DESIGN.md` -- 补充通用剪贴板属于跨设备来源而非来源应用的展示规范，防止后续退化成 AirDrop/Apple Logo 或具体设备猜测。

**Acceptance Criteria:**
- Given 剪贴板来自其他 Apple 设备，when ClipClop 捕获并选中该记录，then metadata 显示“双设备图标 + 其他 Apple 设备”，提示为“通过 Apple 通用剪贴板同步”。
- Given 同一时刻某个本机应用位于前台，when 远程标记存在，then 该应用不会覆盖通用剪贴板来源。
- Given 普通本机应用复制内容，when ClipClop 捕获记录，then 原有应用名称、应用图标和忽略来源操作保持可用。
- Given 通用剪贴板记录被选中，when 用户打开记录操作菜单，then “忽略此来源应用”不可执行。

## Spec Change Log

## Design Notes

专用来源 ID 使用非路径值 `com.apple.universal-clipboard`，只作为内部稳定判别符。双设备图标由前端内置 SVG/CSS 绘制，使用 `currentColor`，尺寸与现有 22px 来源图标一致；文字承担 Apple 生态语义，图形只表达跨设备，避免品牌误导。

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 来源解析测试全部通过。
- `pnpm check` -- Svelte 与 TypeScript 检查无错误。
- `pnpm build` -- 前端生产构建成功。

**Manual checks (if no CLI):**
- 在同一 Apple 账户设备间复制测试文本，确认新记录的名称、双设备图标、提示与禁用忽略操作均正确；再从 Mac 本机应用复制一次，确认原有应用来源未退化。

## Suggested Review Order

**来源检测与优先级**

- 捕获入口先确认远程标记，失败时不再错误猜测本机来源。
  [`system.rs:239`](../../src-tauri/src/clipboard/system.rs#L239)

- 通用剪贴板来源优先于声明、前台及最近活动应用。
  [`system.rs:259`](../../src-tauri/src/clipboard/system.rs#L259)

**界面表达与操作边界**

- 专用来源跳过应用图标查询并保持现有应用链路不变。
  [`+page.svelte:155`](../../src/routes/+page.svelte#L155)

- 双设备图标、名称与可访问说明共同表达跨设备同步。
  [`+page.svelte:769`](../../src/routes/+page.svelte#L769)

- 系统来源不出现在“忽略此来源应用”操作中。
  [`+page.svelte:824`](../../src/routes/+page.svelte#L824)

**规范与回归测试**

- 设计规范锁定命名、图标和禁止猜测的边界。
  [`DESIGN.md:179`](../../DESIGN.md#L179)

- 回归测试覆盖远程优先及普通本机来源保持不变。
  [`system.rs:730`](../../src-tauri/src/clipboard/system.rs#L730)
