---
title: 'Refactor frontend composition'
type: 'refactor'
created: '2026-08-30'
status: 'in-review'
baseline_commit: 'a20f4d93042d878c4b6b28d8836111c6ede76b3c'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/DESIGN.md'
  - '{project-root}/docs/interaction-contract.md'
  - '{project-root}/openspec/changes/refactor-frontend-composition/SPEC.md'
  - '{project-root}/openspec/changes/refactor-frontend-composition/design.md'
  - '{project-root}/openspec/changes/refactor-frontend-composition/tasks.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop 的 feature/API/session 边界健康，但 Settings 与 History 编排组件混入多个独立视图和大块 CSS；一次非法 scoped CSS 已导致整个设置页样式失效。静态桌面 UI 还普遍暴露网页式文本光标和误选择。

**Approach:** 先记录长期前端边界和交互契约，再采用行为冻结式迁移：统一静态文本选择策略，拆出 Settings 的高变化工作流和 History 的标题栏/动作栏，让 CSS 随 DOM owner 移动。继续 feature-first，不引入完整 DDD 或预制 UI kit。

## Boundaries & Constraints

**Always:** Svelte 5/TypeScript 与现有 token；复杂控件继续用 Bits UI、简单控件保持原生；Session/store 状态单 owner 且不接触 DOM；展示组件不直接 raw `invoke()`；CSS 随 DOM owner；`app.css` 只承载 base/token/真正全局策略；每批先有护栏再移动并运行生产构建；除文本选择策略外保持布局、文案、键盘、焦点、ARIA、IPC、更新、预览和平台行为。

**Ask First:** 新依赖、新状态库、改变组件边界导致大量镜像 props/store、修改既有交互或视觉、扩大 Rust/IPC 范围。

**Never:** 完整 frontend DDD/Clean Architecture、Repository/UseCase/DI/event bus；通用 Row/Button/Keycap 或空 UI library；按行数/token 数量机械重构；新的 HistoryView wrapper；重写 HistorySession/PreviewSession/updater store/keyboard/pager；把 scoped CSS 全局化规避问题。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 静态 UI | 标题、列表、设置、菜单、发布记录、meta | 默认箭头且不可拖选 | 不逐组件复制规则 |
| 可编辑文本 | text/search input、textarea、contenteditable | 原生 I-beam、选择与编辑 | checkbox/switch 不进入白名单 |
| 正文预览 | 右侧 text/link `.preview-body.text-preview` | 可选择复制 | meta、路径、颜色、loading 不可选 |
| Settings 拆分 | load/save/rollback、快捷键、更新、release | 行为和状态 owner 不变 | 子组件不复制 async state machine |
| History 拆分 | 菜单、确认、focus、Space preview gate | Escape/focus/action 与现状一致 | 每批可独立回滚 |

</frozen-after-approval>

## Code Map

- `docs/architecture*.md`、`docs/interaction-contract.md` -- 长期边界与文本选择契约。
- `src/app.css` -- 应用级不可选择默认与窄白名单。
- `src/lib/settings/SettingsView.svelte` -- 保留加载/保存/Tabs，迁出 General、Shortcut、Update、Release。
- `src/lib/history/HistoryWorkspace.svelte` -- 保留 session/view/focus/mode，迁出 titlebar 和 actionbar。
- `src/lib/history/keyboard.ts` -- 仅承接可脱离 DOM 的既有决策。

## Tasks & Acceptance

**Execution:**
- [x] 文档和 baseline -- 同步架构/交互契约并记录绿灯。
- [ ] 文本选择 -- 全局默认 + 编辑/正文预览白名单，双平台核对（代码和 macOS 问题修复完成，待 Windows 复核）。
- [x] Settings 护栏与拆分 -- GeneralSettings、ShortcutSettings、UpdateSettings、ReleaseNotes；清理死 CSS/ignore。
- [x] History 护栏与拆分 -- AppTitleBar、HistoryActionBar；集中 open/mode 同步。
- [ ] 完整审计 -- 单向依赖、状态 owner、CSS owner、OpenSpec tasks 与门禁逐项核验（自动化完成，待双平台人工验收）。

**Acceptance Criteria:**
- Given 任一静态 UI，when 鼠标移动或拖动，then 不出现文本 I-beam 且不产生选择。
- Given 文本输入或右侧 text/link 正文，when 编辑或拖选，then 原生选择复制可用。
- Given Settings/History 子视图拆出，when 执行现有键盘、焦点、保存、更新、预览和删除流程，then observable behavior 不变。
- Given 每批提交，when 运行检查，then `pnpm check/test/build` 与 OpenSpec strict validate 通过且无 CSS syntax/minify warning。

## Spec Change Log

## Design Notes

拆分以独立变化原因和窄输入输出为准；出现 props 镜像或无行为 wrapper 时停止。共享 primitive 需三个结构、语义、交互、样式一致的消费者且净减码。

## Verification

**Commands:**
- `pnpm check && pnpm test && pnpm build && git diff --check`
- `openspec validate refactor-frontend-composition --strict`

**Manual checks:**
- macOS/Windows：文本指针与选择白名单、Settings 各分类、History 菜单/确认、浅深主题、键盘/focus、更新和 Preview capability。

**Latest evidence (2026-08-31):**
- `pnpm check`：0 errors / 0 warnings。
- `pnpm test`：20 files / 98 tests passed。
- `pnpm build`：passed，无 CSS syntax/minify warning。
- `git diff --check` 与 `openspec validate refactor-frontend-composition --strict`：passed。
- macOS 实机发现并修复预览空白区文本指针范围，用户复核通过；Windows 因当前无可用设备仍待验证。
