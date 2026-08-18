---
title: 'Trim copied text whitespace setting'
type: 'feature'
created: '2026-08-19'
status: 'done'
baseline_commit: '4f12a61ca46c406c53b47df54fd9e4bb1047bbd9'
context:
  - '{project-root}/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 用户从 ClipClop 复制或粘贴文本时，记录中无意保留的开头、结尾空白会一起进入目标应用，当前只能手工清理。

**Approach:** 在“历史记录”设置中增加一个默认关闭的“去除文本首尾空白”开关。开启后，ClipClop 将历史记录写入系统剪贴板时使用 Rust 原生 `trim()` 清理纯文本 flavor；历史数据本身保持不变，因此已有记录也立即适用。

## Boundaries & Constraints

**Always:** 设置必须兼容旧版已保存 JSON、默认关闭，并沿用现有整份设置保存流程；开关使用现有原生 checkbox/switch、CSS token 和无障碍标注；复制、自动粘贴、手动降级到剪贴板三条结果共用同一处理边界；Unicode 首尾空白由 Rust `str::trim()` 处理。

**Ask First:** 若要裁剪富文本的可见内容、修改已保存历史、或让开启设置后强制丢弃富文本格式，必须先征得用户确认。

**Never:** 不裁剪内部空白；不修改图片、文件、HTML、RTF 或自定义 flavor；不增加数据库迁移、新 IPC 命令、第三方依赖或自定义开关组件。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 开关关闭 | `"  hello \n"` | 原样写入纯文本剪贴板 | 沿用现有剪贴板错误 |
| 开关开启 | `"  hello \n"` | 写入 `"hello"` | 沿用现有剪贴板错误 |
| 内部空白 | `"a  b\nc"` | 内部空格与换行保留 | N/A |
| 全空白文本 | `" \n\t"` | 写入空字符串；其他 flavor 不变 | N/A |
| 富文本/文件/图片 | 同时含非纯文本 flavor | 非纯文本 flavor 原样写入 | 沿用现有格式错误 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/settings/model.rs` -- 持久化设置模型、旧 JSON 默认值和 IPC shape 测试。
- `src-tauri/src/workflows/clip_actions.rs` -- 历史记录复制入口。
- `src-tauri/src/workflows/paste_clip.rs` -- 自动粘贴与剪贴板降级入口。
- `src-tauri/src/clipboard/system.rs` -- 两条入口共享的系统剪贴板写入和 flavor 转换边界。
- `src/lib/settings/api.ts` -- 前端 Settings 类型。
- `src/lib/settings/SettingsView.svelte` -- “历史记录”设置开关。
- `src/lib/i18n/catalogs.ts` -- 中英文开关名称和说明。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/settings/model.rs`、`src/lib/settings/api.ts` -- 增加默认关闭的 `trim_whitespace` 布尔设置，并验证旧设置兼容与 IPC shape。
- [x] `src-tauri/src/workflows/clip_actions.rs`、`src-tauri/src/workflows/paste_clip.rs`、`src-tauri/src/clipboard/system.rs` -- 将已保存设置传入共享写入边界，仅裁剪 `text/plain`，并覆盖开关及边缘输入测试。
- [x] `src/lib/settings/SettingsView.svelte`、`src/lib/i18n/catalogs.ts` -- 复用现有 switch 行加入双语、可访问的历史设置。

**Acceptance Criteria:**
- Given 用户升级且旧设置中没有新字段，when ClipClop 读取设置，then 开关为关闭且其他设置不变。
- Given 用户开启开关，when 从历史记录执行复制、粘贴或粘贴失败降级，then 写入系统剪贴板的纯文本首尾 Unicode 空白被删除。
- Given 用户关闭开关，when 执行相同动作，then 所有 flavor 保持原值。
- Given 记录包含富文本、图片、文件或自定义 flavor，when 开关开启，then 这些非纯文本 flavor 不被修改。

## Spec Change Log

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml settings::model` -- expected: 新设置兼容测试通过。
- `cargo test --manifest-path src-tauri/Cargo.toml clipboard::system` -- expected: 裁剪行为测试通过。
- `pnpm check` -- expected: Svelte 与 TypeScript 检查通过。
- `pnpm test -- --run` -- expected: 前端测试通过。

**Manual checks (if no CLI):**
- 在“历史记录”设置中确认开关可聚焦、可切换、保存后重开仍保留；分别复制带首尾空白的纯文本并验证开关两种状态。

## Suggested Review Order

**共享写入边界**

- 所有复制与粘贴在此仅裁剪纯文本 flavor。
  [`system.rs:26`](../../src-tauri/src/clipboard/system.rs#L26)

- 复制入口传递设置，同时保留原有置顶行为。
  [`clip_actions.rs:56`](../../src-tauri/src/workflows/clip_actions.rs#L56)

- 自动粘贴与失败降级复用相同写入语义。
  [`paste_clip.rs:26`](../../src-tauri/src/workflows/paste_clip.rs#L26)

**设置与界面**

- 新字段默认关闭并通过 serde 兼容旧设置。
  [`model.rs:21`](../../src-tauri/src/settings/model.rs#L21)

- 原生 switch 复用现有样式与无障碍关联。
  [`SettingsView.svelte:404`](../../src/lib/settings/SettingsView.svelte#L404)

- 前端类型保持 IPC 契约一致。
  [`api.ts:11`](../../src/lib/settings/api.ts#L11)

**验证**

- 单测覆盖关闭、Unicode 裁剪、内部空白和非纯文本保留。
  [`system.rs:383`](../../src-tauri/src/clipboard/system.rs#L383)
