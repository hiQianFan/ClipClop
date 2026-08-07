---
title: 'macOS 文件预览权限与无弹窗降级'
type: 'feature'
created: '2026-08-04'
status: 'done'
baseline_commit: '58881b6fa7a6f0fcda7ae7693878c27c1d66b21e'
context:
  - 'docs/architecture.md'
  - 'PRIVACY.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop 对文件条目执行详情读取或 Quick Look 时会直接访问原始路径，macOS 可能反复弹出受保护目录权限请求，破坏剪贴工作流。

**Approach:** macOS 文件内容预览增加持久化显式开关并默认关闭；Settings 与快速入门提供完整磁盘访问快捷入口。未开启时前后端都禁止读取原文件，仅展示数据库已有的文件名或路径、来源应用和类型；用户完成系统授权并主动开启后复用现有预览。

## Boundaries & Constraints

**Always:** 默认关闭；Settings 与快速入门均可直达 macOS 完整磁盘访问设置；授权步骤可跳过；未开启时不得执行文件存在性检查、metadata、图片解码、文件资源加载或 Quick Look；Rust 共享入口必须二次门控；非文件类型和 Windows 行为不变；文案明确系统授权由用户手动完成且可撤销。

**Ask First:** 若现有 Tauri opener 无法打开对应系统设置页面；若可靠实现必须引入新依赖、数据库 schema 迁移或改变文件捕获格式；若必须在未开启状态读取任意原文件才能展示现有基本信息。

**Never:** 自动授予或绕过 TCC、请求管理员密码、静默探测完整磁盘访问、复制原文件作为缓存、引入逐文件/文件夹选择授权、改变辅助功能权限流程、提交或推送代码。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 默认降级 | macOS，文件预览开关关闭，选择或按 Space 处理文件条目 | 仅显示已有基本信息，不调用文件资源或 Quick Look，不弹系统权限框 | 显示“文件预览未开启”的非阻塞提示 |
| 主动授权 | 用户从 Settings 或快速入门点击权限入口 | 打开完整磁盘访问系统设置；快速入门可继续或跳过 | 打开失败时显示本地化错误，开关保持关闭 |
| 授权后预览 | 用户手动授权并开启应用内开关 | 显式文件预览沿用现有 Quick Look/资源加载 | 系统实际仍拒绝时按不可预览降级，不自动重试 |
| 非文件内容 | 任意文件预览开关状态 | 文本、图片、链接、颜色和代码预览保持现状 | 沿用现有错误处理 |
| Windows | 任意设置状态 | 不显示 macOS 权限入口，现有行为不变 | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/settings.rs` -- 持久化设置模型与向后兼容默认值。
- `src-tauri/src/commands/settings.rs` -- 打开系统设置的受控原生命令。
- `src-tauri/src/commands/preview.rs` -- 文件资源与原生预览的共享安全边界。
- `src/lib/settings/api.ts`、`src/lib/settings/SettingsView.svelte` -- 设置类型、开关及权限快捷入口。
- `src/lib/onboarding/api.ts`、`src/lib/onboarding/OnboardingView.svelte` -- 快速入门权限入口及可跳过交互。
- `src/lib/history/HistoryWorkspace.svelte` -- 未启用状态下的文件基本信息降级与请求抑制。
- `src/lib/i18n/catalogs.ts` -- 中英文权限、状态和错误文案。
- `PRIVACY.md`、`PRIVACY.zh-CN.md` -- 可选文件访问权限与降级行为说明。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/settings.rs`、`src/lib/settings/api.ts` -- 增加默认关闭且旧设置兼容的文件预览开关。
- [x] `src-tauri/src/commands/settings.rs`、`src-tauri/src/lib.rs`、`src/lib/onboarding/api.ts` -- 复用现有命令模式打开 macOS 完整磁盘访问设置。
- [x] `src-tauri/src/commands/preview.rs` -- 在所有原文件读取入口统一拒绝未启用的文件预览，并将设置锁持有到读取结束。
- [x] `src/lib/settings/SettingsView.svelte`、`src/lib/onboarding/OnboardingView.svelte` -- 增加 macOS 专属、可跳过的授权与确认入口。
- [x] `src/lib/history/HistoryWorkspace.svelte` -- 未启用时停止文件资源请求并呈现基本信息提示。
- [x] `src/lib/i18n/catalogs.ts`、隐私说明 -- 补齐双语用户文案。
- [x] 相关 Rust/TypeScript/Svelte 测试 -- 覆盖默认值、旧数据、命令门控和 UI 请求行为。

**Acceptance Criteria:**
- Given 全新安装或旧版本升级且未主动启用，when 用户浏览、选择或预览受保护目录的文件条目，then ClipClop 不读取原文件且不触发文件权限弹窗。
- Given 用户拒绝或跳过授权，when 完成快速入门并继续使用，then 核心剪贴功能可用且文件基本信息可见。
- Given 用户完成系统授权并开启文件预览，when 显式预览文件，then 现有原生预览正常工作。
- Given 非文件剪贴内容或 Windows 环境，when 使用预览，then 行为与改动前一致。

## Spec Change Log

- 2026-08-04：审查发现命令检查与异步文件读取间存在竞态，同时规格误列未修改的 workflow 文件。门控任务改为要求持锁覆盖实际读取，并纠正 Code Map；保留前后端双门控、默认关闭和 Windows 不变。
- 2026-08-05：应用户反馈重构授权交互（未改动 frozen 意图/边界）。系统设置入口按钮改为纯跳转、文案固定、不记录或展示任何状态；应用内启用改由标准 Switch 开关承担（Settings 随保存落盘、快速入门即时落盘）。移除易失的“已打开设置→确认启用”双阶段流程。快速入门中的文件预览由第 3 步附属块升级为独立第 4 步（仅 macOS，Windows 保持三步）。前后端门控、默认关闭、`file_preview_enabled` 语义不变。
- 2026-08-05（意图重协商 A，随即被 B 取代）：曾一度移除整个应用内门控、只留「管理」跳转，把可用性完全交给系统权限。实测发现：未授权时浏览/选中受保护目录（桌面/文稿/下载）的文件条目会触发 macOS 权限弹窗，反复打断剪贴流程——正是 frozen spec 要消除的问题。故废弃此方案。同轮的 layout 统一（见 DESIGN.md「Settings Row」）保留。
- 2026-08-05（意图重协商 B，最终）：为“默认不打扰”，**恢复应用内 Switch 门控**并回归 frozen 边界：默认关闭时前后端都不读原文件、零弹窗，只显示基本信息并提示去开启。那一栏定型为「打开完整磁盘访问」跳转按钮 + Switch 开关的控件组（Settings 随保存落盘、快速入门即时落盘）。恢复 `file_preview_enabled` 字段/命令/`preview.rs` 双门控与设置锁/前端状态/`ClipPreview` 占位。新增**读失败静默降级**：Switch 开但系统未授权时，文件读取失败不再报红，而是回落到基本信息占位并引导去授权——避免用户没按“先授权再开关”顺序时被系统弹窗打断。macOS 无法内联授权 FDA、也无法在不试读的情况下探测授权状态，故采用“Switch 表意愿 + 按钮引导 + 失败降级”组合，而非探测式流程。

## Design Notes

文件预览由两道门控叠加：应用内 Switch（`file_preview_enabled`，默认关）决定 app 是否触碰原文件，macOS 完整磁盘访问权限决定系统是否放行读取。Switch 关时前后端都不读，浏览过程零系统弹窗，仅显示基本信息并提示去开启——这是“默认不打扰”的关键。用户开启 Switch 表示显式意愿；「打开完整磁盘访问」按钮把用户引导到系统设置手动授权（macOS 无内联授权、也不可探测授权状态，故不做探测式分支）。若用户未按“先授权 FDA 再开开关”的顺序，读取失败会静默降级为基本信息 + 引导，而不是报错或让系统弹窗打断。Settings 行布局遵循 DESIGN.md 的 Settings Row 两区契约，文件预览那一栏是「按钮 + 开关」控件组。

## Verification

**Commands:**
- `pnpm check` -- Svelte/TypeScript 检查通过。
- `pnpm test -- --run` -- 前端相关测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 设置与预览门控测试通过。
- `git diff --check` -- 无格式错误。

**Manual checks:**
- 在未授权 macOS 上复制 Desktop、Documents、Downloads 文件，逐项执行选择、详情和 Space，确认无权限弹窗；随后通过两个入口完成授权并验证 Quick Look。

**Recorded evidence:**
- `2026-08-04`：完整质量门通过；前端 43 项、Rust 51 项测试通过，Svelte/TypeScript、production build、rustfmt、clippy、diff check 均通过。
- `2026-08-04`：实机打开完整磁盘访问深链，System Settings 窗口标题确认为“完全磁盘访问权限”；未自动修改任何系统权限。
- 禁用路径由命令级操作抑制测试和设置锁并发测试证明不会进入原文件操作；未使用真实剪贴板或私人路径作为测试数据。

## Suggested Review Order

**核心安全边界**

- 设置锁覆盖检查与读取，关闭动作不会穿过竞态窗口。
  [`preview.rs:14`](../../src-tauri/src/commands/preview.rs#L14)

- 默认关闭且旧设置反序列化自动降级。
  [`settings.rs:17`](../../src-tauri/src/settings.rs#L17)

- 系统设置深链与独立持久化命令保持窄权限边界。
  [`settings.rs:51`](../../src-tauri/src/commands/settings.rs#L51)

**无权限降级**

- 前端读取存储开关，非 macOS 保持原有行为。
  [`HistoryWorkspace.svelte:86`](../../src/lib/history/HistoryWorkspace.svelte#L86)

- 文件选择和 Space 在关闭时不发送原文件请求。
  [`HistoryWorkspace.svelte:199`](../../src/lib/history/HistoryWorkspace.svelte#L199)

- 基本信息继续显示，并明确提示未访问原文件。
  [`ClipPreview.svelte:42`](../../src/lib/history/ClipPreview.svelte#L42)

**授权入口**

- Settings 用纯跳转按钮 + 独立 Switch，按钮不记录/展示状态，开关随保存落盘。
  [`SettingsView.svelte:142`](../../src/lib/settings/SettingsView.svelte#L142)

- 快速入门第 4 步（仅 macOS）复用同一跳转按钮 + 即时落盘的 Switch，可跳过。
  [`OnboardingView.svelte:184`](../../src/lib/onboarding/OnboardingView.svelte#L184)

**文案与证据**

- 双语文案说明可选、可撤销和关闭后的行为。
  [`catalogs.ts:41`](../../src/lib/i18n/catalogs.ts#L41)

- 隐私说明记录实际文件访问边界。
  [`PRIVACY.md:7`](../../PRIVACY.md#L7)

- 命令级测试证明禁用时操作闭包不会执行。
  [`preview.rs:143`](../../src-tauri/src/commands/preview.rs#L143)
