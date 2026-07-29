---
title: 'ClipClop clipboard history major refactor'
type: 'refactor'
created: '2026-07-29'
status: 'in-progress'
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
- [ ] `openspec/changes/refactor-clipboard-history-architecture/` -- 完成全部 Architecture tasks，行为冻结并提交。
- [ ] `openspec/changes/harden-ipc-boundary/` -- 完成全部 IPC Hardening tasks、before→after 测试并提交。
- [ ] `openspec/changes/stabilize-keyboard-focus/` -- 完成全部 Keyboard Focus tasks、可访问性与焦点测试并提交。
- [ ] `src-tauri/`、`src/` -- 运行完整 Rust/TypeScript/Svelte 测试、lint、构建和跨平台可执行检查。

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
- `cargo fmt --check && cargo clippy --all-targets --all-features -- -D warnings && cargo test` -- Rust 全绿。
- `pnpm check && pnpm test && pnpm build` -- Svelte/TypeScript 全绿。
- `openspec validate <change> --strict` -- 三个 change 均有效。

**Manual checks:**
- macOS 与 Windows 验证召回、翻页、搜索、菜单、复制/粘贴、删除、Preview/Quick Look、Settings 与重新召回。
