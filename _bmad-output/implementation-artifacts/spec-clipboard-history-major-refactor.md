---
title: 'ClipClop clipboard history major refactor'
type: 'refactor'
created: '2026-07-29'
status: 'done'
baseline_commit: '56775d4'
context:
  - 'openspec/changes/refactor-clipboard-history-architecture/'
  - 'openspec/changes/harden-ipc-boundary/'
  - 'openspec/changes/stabilize-keyboard-focus/'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Clipboard History 的 IPC、跨能力编排、Preview/Settings 生命周期和前端交互状态仍耦合在 Command 与 `+page.svelte` 中，导致焦点不连续、平台能力边界过宽，并使后续维护容易产生循环依赖和部分成功状态。

**Approach:** 严格按三个已批准 OpenSpec change 依次实施：先行为冻结地建立 History/Preview/Workflow/Session 边界，再加固 IPC 与并发失败语义，最后落地键盘焦点模型；每阶段独立测试和提交。

## Boundaries & Constraints

**Always:** Architecture 阶段保持数据库、IPC 形状、错误、副作用和现有缺陷行为不变；只有满足跨能力/顺序/补偿/并发不变量门槛的操作使用 Workflow；组件无裸 `invoke()`；每阶段完成 OpenSpec tasks、自动化测试和对应提交。

**Ask First:** 数据库 schema 迁移、新依赖、删除用户数据、无法保持现有持久记录兼容、需要改变三个已批准提案之外的用户行为。

**Never:** Command Bus、DI 容器、单实现 trait/repository、通用 job system、为简单查询增加 pass-through Workflow、把焦点或 DOM 放入 HistorySession、将平台线程亲和 API 机械放入 `spawn_blocking`。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|---------------|----------------------------|----------------|
| Existing database | 当前 `clipclop.db` | 三阶段后记录无损可读 | 阻止完成并保留原库 |
| Delete/cache race | Preview 生成与删除重叠 | Hardening 后无孤儿缓存 | 清理失败保留记录并返回错误 |
| Settings overlap | 两次更新或更新时间戳重叠 | 串行、补偿、时间戳不丢失 | 补偿失败明确报告并可启动校正 |
| Keyboard context | 翻页、空白 Preview、菜单、Quick Look | Selection 保留且焦点回到明确目标 | stale 完成不得抢焦点 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/` -- Tauri IPC adapters。
- `src-tauri/src/{clips,clipboard,paste,settings,state}.rs` -- 当前待迁移边界。
- `src-tauri/src/workflows/` -- 新增的稀疏应用编排层。
- `src-tauri/src/{history,preview}/` -- History 与派生资源能力。
- `src/routes/+page.svelte` -- 当前前端耦合入口。
- `src/lib/history/` -- 新 History API、Session 与组件。
- `openspec/changes/*/tasks.md` -- 每阶段权威任务清单。

## Tasks & Acceptance

**Execution:**
- [x] `openspec/changes/refactor-clipboard-history-architecture/` -- Architecture 代码、自动化任务和严格校验完成并提交。
- [x] `openspec/changes/harden-ipc-boundary/` -- IPC Hardening 代码、自动化测试和严格校验完成并提交。
- [x] `openspec/changes/stabilize-keyboard-focus/` -- Keyboard Focus 代码、可访问性测试和严格校验完成并提交。
- [x] `src-tauri/`、`src/` -- Rust/TypeScript/Svelte 测试、lint 和生产构建全部通过。
- [ ] macOS 与 Windows 真机验收、平台失败注入和代表性延迟测量。

**Acceptance Criteria:**
- Given 现有用户数据库和设置，when 启动重构后的应用，then 历史记录、设置、复制粘贴和面板生命周期保持安全可用。
- Given 任一 Tauri Command，when 检查其实现，then 它只适配 IPC 并调用一个直接 capability 或合格 Workflow。
- Given History 中存在 selection，when 用户使用指针、翻页、菜单、Search、文件 Tab 或 Quick Look，then selection/focus/mode 遵循 keyboard-focus spec。
- Given 三份 change，when 执行严格校验与任务审计，then 所有任务有实现和测试证据。

## Spec Change Log

## Design Notes

详细设计以三份 OpenSpec 为唯一权威来源；本文件只记录跨 change 的实施顺序、共同边界和完成证据，避免重复规格漂移。

## Verification

**Commands:**
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` -- 38 个 Rust 测试通过。
- `pnpm check && pnpm test && pnpm build` -- 0 个诊断、36 个前端测试通过、生产构建成功。
- `openspec validate <change> --strict` -- 三个 change 均通过严格校验。
- `git diff 56775d4 -- src-tauri/schema.sql` -- 无数据库 schema 变化。

**Manual checks:**
- macOS 与 Windows 验证召回、翻页、搜索、菜单、复制/粘贴、删除、Preview/Quick Look、Settings 与重新召回。

## Suggested Review Order

**入口与边界**

- 从组合根查看能力依赖和状态所有权。
  [`state.rs:8`](../../src-tauri/src/state.rs#L8)

- Command 只传输参数并进入意图级用例。
  [`history.rs:23`](../../src-tauri/src/commands/history.rs#L23)

- 页面入口只负责挂载 History 工作区。
  [`+page.svelte:2`](../../src/routes/+page.svelte#L2)

**跨能力一致性**

- 删除先协调派生缓存，再提交 History 删除。
  [`clip_actions.rs:7`](../../src-tauri/src/workflows/clip_actions.rs#L7)

- Settings 更新串行执行并补偿外部副作用。
  [`settings_update.rs:19`](../../src-tauri/src/workflows/settings_update.rs#L19)

- Preview 发布在生命周期锁内重验实体并原子替换。
  [`mod.rs:186`](../../src-tauri/src/preview/mod.rs#L186)

- 原生预览结果由单一 typed workflow 穷尽表达。
  [`preview_clip.rs:19`](../../src-tauri/src/workflows/preview_clip.rs#L19)

**交互状态与焦点**

- Session 集中持有查询、选择、缓存和异步失效版本。
  [`session.svelte.ts:20`](../../src/lib/history/session.svelte.ts#L20)

- 明确模式隔离 Browse、Search、菜单、确认和文件标签。
  [`HistoryWorkspace.svelte:15`](../../src/lib/history/HistoryWorkspace.svelte#L15)

- List 自持键盘浏览，避免全局焦点路由。
  [`HistoryWorkspace.svelte:406`](../../src/lib/history/HistoryWorkspace.svelte#L406)

- 列表项暴露全局集合位置和选中语义。
  [`HistoryList.svelte:82`](../../src/lib/history/HistoryList.svelte#L82)

**回归证据**

- Session 测试覆盖 stale、分页、后继选择和缓存失效。
  [`session.test.ts:32`](../../src/lib/history/session.test.ts#L32)

- 并发测试证明删除或清空获胜后无法发布缓存。
  [`mod.rs:371`](../../src-tauri/src/preview/mod.rs#L371)

- SSR 组件测试锁定列表 ARIA 集合元数据。
  [`HistoryList.test.ts:6`](../../src/lib/history/HistoryList.test.ts#L6)
