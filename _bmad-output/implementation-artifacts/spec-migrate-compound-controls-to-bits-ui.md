---
title: 'Migrate compound controls to Bits UI'
type: 'refactor'
created: '2026-08-20'
status: 'done'
baseline_commit: '6917c9e735cd12b40109a3663d97e9e49b10efb6'
context:
  - 'openspec/changes/migrate-compound-controls-to-bits-ui/proposal.md'
  - 'openspec/changes/migrate-compound-controls-to-bits-ui/design.md'
  - 'openspec/changes/migrate-compound-controls-to-bits-ui/specs/frontend-interaction-primitives/spec.md'
  - 'docs/interaction-contract.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop 已有稳定的键盘、焦点和 ARIA 行为，但菜单、Tabs、破坏性确认等复合控件仍手工维护通用交互，使业务组件承担重复的导航、焦点恢复和 dismiss 逻辑。

**Approach:** 以锁定的 `bits-ui@2.18.1` 迁移所有具有匹配 primitive 且可无损适配的复合控件；业务状态、快捷键、焦点落点、Escape 层级、样式和平台行为保持不变。

## Boundaries & Constraints

**Always:** 以 OpenSpec 和 `docs/interaction-contract.md` 为行为基线；使用受控 primitive 直连现有状态；保留现有 class、CSS token、DOM 布局与 reduced-motion；每类 primitive 独立验证和回滚；迁移前建立真实 DOM 焦点测试。

**Ask First:** AlertDialog 无法保持 footer 内联交互；Pagination 与 scrubber/异步刷新冲突；需要改变既有焦点、快捷键、视觉或新增非测试依赖时停止并请求决定。

**Never:** 不迁移三个常驻 Listbox；不包装普通按钮、输入框、原生 Select/Switch/滚动区；不引入 Tailwind、shadcn-svelte、镜像状态或通用 UI 包装层；不升级 Bits UI；不改 IPC、后端、数据库或文案。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Layer exit | 菜单或确认打开时按 Escape | 只关闭当前层并恢复契约焦点 | 已处理事件不再关闭窗口 |
| Trigger gone | 确认关闭时 invoker 已卸载 | 焦点回到 HistoryList | 不聚焦失效节点 |
| File tab edge | 首/末文件按越界方向键 | 保持当前文件，不循环 | 不重复加载预览 |
| Indeterminate progress | `updateProgress === null` | 暴露不确定 progressbar 并保持动画 | 不伪造百分比 |
| Pagination conflict | primitive 导致重复刷新或状态竞争 | 回退 Pagination 适配，保留自定义分页 | 记录阻断原因 |

</frozen-after-approval>

## Code Map

- `src/lib/history/HistoryWorkspace.svelte` -- 应用/操作菜单、删除确认和窗口交互模式
- `src/lib/history/ClipPreview.svelte` -- 多文件 Tabs
- `src/lib/history/HistoryList.svelte` -- 自定义 Listbox 与条件迁移的分页 scrubber
- `src/lib/settings/SettingsView.svelte` -- 设置 Tabs、清空确认和更新进度
- `src/lib/onboarding/OnboardingView.svelte` -- 语言菜单与保留的练习 Listbox
- `src/lib/history/keyboard.ts` -- Escape 分层路由契约
- `openspec/changes/migrate-compound-controls-to-bits-ui/tasks.md` -- 分阶段任务清单

## Tasks & Acceptance

**Execution:**
- [x] 测试配置与相关 `*.test.ts` -- 增加最小 DOM 行为护栏，覆盖菜单、Tabs、确认和 Progress
- [x] `HistoryWorkspace.svelte` -- 迁移两个 DropdownMenu 和历史 AlertDialog，删除被接管的手写逻辑
- [x] `OnboardingView.svelte` -- 迁移语言 RadioGroup menu，保留特殊打开焦点
- [x] `SettingsView.svelte` -- 迁移垂直 Tabs、清空 AlertDialog 和 Progress
- [x] `ClipPreview.svelte` -- 迁移水平非循环文件 Tabs，保留预览加载和 Escape
- [x] `HistoryList.svelte` -- 验证 Pagination；命中零减码停止条件并保留现状
- [x] OpenSpec tasks -- 同步完成状态并执行全部自动门禁

**Acceptance Criteria:**
- Given 任一迁移控件，when 使用既有鼠标或键盘路径，then 业务结果、焦点目的地、ARIA 和视觉与迁移前一致。
- Given 菜单、Tabs 或确认层处理按键，when 事件到达窗口路由，then 不发生第二次处理。
- Given macOS 或 Windows，when 运行对应功能，then 平台分支和系统能力流程保持不变。
- Given 无匹配 primitive 的列表或简单控件，when 完成迁移，then 其实现未被强行包装。

## Spec Change Log

## Design Notes

优先直接使用 Bits UI compound API。菜单保留当前 DOM containing block；Tabs 显式设置 orientation、activationMode 和 loop；AlertDialog 先验证无 Portal/Overlay 与无 scroll lock 的内联行为。只有出现两处完全相同且稳定的结构后才考虑提取包装。

## Verification

**Commands:**
- `pnpm check` -- Svelte 与 TypeScript 无错误
- `pnpm test` -- 全部单元/组件测试通过
- `pnpm build` -- 生产构建通过
- `git diff --check` -- 无格式错误
- `openspec validate migrate-compound-controls-to-bits-ui --strict` -- OpenSpec 严格校验通过

**Manual checks:**
- macOS/Windows 的浅色、深色、125%/150% 缩放和 reduced-motion 下走查键盘与视觉。

## Suggested Review Order

**菜单与退出层级**

- 受控菜单保留互斥状态、Escape 层级与各自焦点归还。
  [`HistoryWorkspace.svelte:580`](../../src/lib/history/HistoryWorkspace.svelte#L580)

- 语言菜单保留当前项、末项打开以及 Tab 自然退出。
  [`OnboardingView.svelte:271`](../../src/lib/onboarding/OnboardingView.svelte#L271)

**Tabs 与确认层**

- 设置 Tabs 保留垂直循环和跨区域焦点契约。
  [`SettingsView.svelte:371`](../../src/lib/settings/SettingsView.svelte#L371)

- 文件 Tabs 显式非循环，并复用现有预览加载回调。
  [`ClipPreview.svelte:65`](../../src/lib/history/ClipPreview.svelte#L65)

- 内联 AlertDialog 保留确认初始焦点和 invoker 回退。
  [`HistoryWorkspace.svelte:645`](../../src/lib/history/HistoryWorkspace.svelte#L645)

- 设置确认与 Progress 只替换通用交互和 ARIA 语义。
  [`SettingsView.svelte:452`](../../src/lib/settings/SettingsView.svelte#L452)

**回归护栏**

- DOM 集成测试覆盖菜单、Tabs、对话框和两类进度。
  [`BitsInteractionHarness.test.ts:6`](../../src/lib/ui/BitsInteractionHarness.test.ts#L6)

- 真实快速入门组件覆盖特殊打开焦点与 Tab 退出。
  [`OnboardingView.test.ts:14`](../../src/lib/onboarding/OnboardingView.test.ts#L14)

- 真实文件预览组件覆盖边界、单次激活和 Escape。
  [`ClipPreview.test.ts:21`](../../src/lib/history/ClipPreview.test.ts#L21)
