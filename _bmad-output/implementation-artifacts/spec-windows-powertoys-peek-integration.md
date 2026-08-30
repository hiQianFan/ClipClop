---
title: 'Integrate optional PowerToys Peek previews on Windows'
type: 'feature'
created: '2026-08-30'
status: 'done'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/DESIGN.md'
  - '{project-root}/_bmad-output/specs/spec-windows-powertoys-peek-integration/SPEC.md'
  - '{project-root}/_bmad-output/specs/spec-windows-powertoys-peek-integration/implementation-details.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Windows 当前把 Space “预览”回退为默认应用打开文件，抢占焦点并中断剪贴板工作流；ClipClop 又不应承担自建或捆绑文件渲染器的成本。

**Approach:** 把 Microsoft PowerToys Peek 作为 Windows 文件预览的可选前置能力：实时检测官方默认安装位置，只有能力可用且当前记录是真实文件时才显示并响应预览；缺失时完全不接管 Space，并在 General 设置提供官方安装说明入口。

## Boundaries & Constraints

**Always:** 检测、权限判断和启动在 Rust 后端完成；仅检查 `%LOCALAPPDATA%` 与 `%ProgramFiles%` 下 PowerToys 官方默认 Peek 路径；使用参数化 `Command` 启动数据库记录解析出的真实文件路径；每次后端预览重新校验能力；主面板和 Quick 每次显示时刷新同一 capability；Windows 普通权限、ready、File 三项同时成立才接管 Space；设置页仅显示 ready/not-installed/elevated 状态和 Microsoft 官方说明链接；macOS Quick Look 及其 `PreviewState` 保持不变；中英文同步；不记录文件路径。

**Ask First:** 新增依赖、扩大安装路径搜索、增加新的 preview provider、改变 macOS 预览生命周期。

**Never:** 不回退默认应用；不捆绑、下载、静默安装或剥离 `PowerToys.Peek.UI.exe`；不接受前端提供 executable/file path；不为非文件内容生成临时文件；不新增数据库偏好；不在 Quick 面板做安装教育；不以 disabled 菜单项或重复 toast 暗示不可用能力。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 未安装 | Windows, candidates absent | 设置显示官方安装入口；两面板不显示预览且 Space 无副作用 | 后端返回 `NotPreviewable` |
| 已就绪 | 普通权限且 Peek 存在，当前项为 File | 主面板与 Quick 可用 Space/菜单启动 Peek | spawn 失败显示现有 inline error，不 fallback |
| 提升权限 | Peek 存在但 ClipClop elevated | 设置解释需普通权限；预览不可用 | 检测失败按 unavailable 并记录无路径 warning |
| 能力变化 | ClipClop 运行期间安装/移除 Peek | 再次呼出任一面板或进入 General 后刷新 | 后端最终校验防止缓存误用 |
| 文件无效 | 非 File、索引无效、路径已删除 | 不接管 Space、不启动外部程序 | 返回 `NotPreviewable` |
| macOS | 任意既有可预览内容 | Quick Look toggle 与关闭行为保持现状 | 既有测试继续通过 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/preview/platform.rs` -- Windows 默认路径、elevation、capability 与 Peek spawn。
- `src-tauri/src/preview/mod.rs`、`workflows/preview_clip.rs` -- 安全解析当前文件并移除默认应用 fallback。
- `src-tauri/src/commands/preview.rs`、`lib.rs`、`Cargo.toml` -- capability IPC、命令注册和 Windows security feature。
- `src/lib/history/api.ts` -- capability DTO 与移除 `FallbackOpened` wire value。
- `src/lib/history/HistoryWorkspace.svelte` -- 主面板刷新、Space 和菜单门控。
- `src/lib/history/QuickPanel.svelte`、`quick-keyboard.ts` -- Quick 显示刷新与 File/Space 路由。
- `src/lib/settings/SettingsView.svelte`、`i18n/catalogs.ts` -- Windows General 集成状态与官方链接。
- preview/history/quick tests -- 路径选择、fallback 移除、capability 和键盘门控。

## Tasks & Acceptance

**Execution:**
- [x] Rust preview 边界 -- 实现 capability、elevation、固定候选检测、File 校验与参数化 Peek 启动，彻底删除默认应用 fallback。
- [x] IPC 与窗口事件 -- 注册 capability 命令，并为主面板/Quick 每次显示发送刷新事件。
- [x] 主面板与 Quick -- 共享 capability，只有 ready + File 时显示入口并接管 Space。
- [x] Windows 设置与 i18n -- General 普通设置行展示 ready/not-installed/elevated/detection-failed 和官方链接。
- [x] 自动化验证 -- 覆盖 capability/Space 路由、既有 macOS 行为和生产构建；Windows target 因本机未安装该 Rust target 留待 Windows CI 验证。

**Acceptance Criteria:**
- Given 未安装 Peek，when Windows 用户在两面板按 Space，then 不调用默认应用、无 toast、无焦点打断。
- Given Peek ready 且当前记录是存在的文件，when 按 Space 或选择预览菜单，then 参数化启动 Peek 且不暴露路径到前端。
- Given 用户运行期间安装 Peek，when 再次呼出主面板或 Quick，then 无需重启 ClipClop 即可识别。
- Given ClipClop elevated、能力检测失败或文件失效，when 尝试预览，then 后端返回不可预览且不 fallback。
- Given macOS 构建，when 使用 Space/Escape，then Quick Look 生命周期与现状一致。

## Spec Change Log

- 2026-08-30: Implemented the approved Windows PowerToys Peek integration and removed the default-app preview fallback.

## Design Notes

Capability wire model：`provider = macos_quicklook | powertoys_peek | unavailable`，`reason = null | not_installed | elevated | detection_failed`。设置行沿用 68px 两区布局，不使用 Microsoft 品牌色或卡片；未安装按钮文案为“了解并安装”，只打开 `https://learn.microsoft.com/windows/powertoys/install`。

## Verification

**Commands:**
- `pnpm test && pnpm check` -- 前端门控、组件和类型通过。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 格式与测试通过。
- `cargo check --manifest-path src-tauri/Cargo.toml --target x86_64-pc-windows-msvc` -- Windows 条件代码可编译。
- `git diff --check` -- 补丁格式正确。

**Manual checks (if no CLI):**
- Windows 真机覆盖未安装、用户级安装、机器级安装、管理员启动、安装后无需重启、文件删除和 spawn 失败。
