---
title: 'Redesign software update status layout'
type: 'feature'
created: '2026-08-30'
status: 'done'
baseline_commit: '6db1575607f375d9c03978bc516d352941492410'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/DESIGN.md'
  - '{project-root}/_bmad-output/planning-artifacts/ux-designs/ux-ClipClop-2026-08-30/.working/demo-update-actions.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 软件更新页把检查结果与下载操作拆成多个区域，按钮过宽、状态切换会改变布局，中间信息与操作之间还会出现无意义空白；重复版本与动作说明削弱了信息层级。

**Approach:** 按已确认 Demo 将更新设置与实时状态组织为同一语义 Section 的两条固定行：第一行负责自动检查、当前版本和手动刷新；第二行使用单行状态轨道展示状态、进度与当前阶段动作，下方版本记录保持主体地位。

## Boundaries & Constraints

**Always:** 复用现有 Svelte store、Bits UI Progress、原生按钮与 CSS token；状态轨道高度固定，页面下方内容不得因状态切换移动；刷新入口位于自动检查行；“跳过此版本”直接显示于第二行且只在版本尚未安装并可放弃时出现；下载中只显示取消；下载完成显示跳过与安装并重启；错误只将失败标题标红，辅助说明保持普通颜色；技术错误继续后台记录；中英文文案同步。

**Ask First:** 改变更新状态机、自动检查频率、安装/重启语义或新增依赖。

**Never:** 不恢复独立更新卡片；不使用 More 菜单承载跳过；不在更新状态栏暴露诊断日志；不重复显示当前版本、产品名或按钮已表达的结果；不改后端下载与安装实现。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 已是最新 | `current` | 显示结果与上次检查时间；无版本动作 | 刷新仍在第一行可用 |
| 发现更新 | update available | 显示目标版本、跳过和下载 | 不生成额外卡片 |
| 下载中 | progress known/unknown | 标题、进度、百分比、取消同一行 | 不显示跳过或刷新；内容不重叠 |
| 已下载 | `downloaded` | 显示跳过与安装并重启 | 跳过会丢弃安装包并标记版本 |
| 下载/安装失败 | error source before install success | 失败标题、简短恢复信息、跳过与重试 | 原始错误只写日志 |
| 重启失败 | install succeeded, relaunch failed | 显示更新已安装与重启 | 不允许跳过或重新安装 |

</frozen-after-approval>

## Code Map

- `src/lib/settings/SettingsView.svelte` -- 更新页结构、状态派生、按钮显示和 CSS。
- `src/lib/i18n/catalogs.ts` -- 中英文更新状态、设置与辅助文案。
- `src/lib/updater/store.svelte.ts` -- 现有状态与错误来源；仅作为 UI 输入，不改变行为。
- `src/lib/settings/SettingsView.test.ts` -- 更新状态 DOM、动作可见性与固定结构验证。

## Tasks & Acceptance

**Execution:**
- [x] `src/lib/settings/SettingsView.svelte` -- 合并检查行与更新卡片为两行 Section，按状态渲染单行轨道与紧凑动作。
- [x] `src/lib/i18n/catalogs.ts` -- 去重并更新中英文文案，不暴露内部限流或模糊诊断信息。
- [x] `src/lib/settings/SettingsView.test.ts` -- 覆盖主要状态的标题、进度、跳过/取消/安装/重启动作。

**Acceptance Criteria:**
- Given 任意更新状态，when 状态切换，then 状态轨道与版本记录位置保持不变且可见内容不重叠。
- Given 用户手动检查，when 点击第一行刷新按钮，then 复用现有检查逻辑且忙碌时不可重复触发。
- Given 目标版本尚未安装且任务不忙，when 状态允许放弃，then 第二行直接显示“跳过此版本”；安装成功后不再显示。
- Given 更新失败，when 页面渲染错误，then 只有失败标题使用危险色且用户获得明确的恢复动作。

## Spec Change Log

## Design Notes

已确认结构：`软件更新标题 → 自动检查/当前版本/刷新 → 单行状态轨道 → 版本记录`。第二行左侧信息组采用内容驱动布局，右侧只按实际动作占宽；不为隐藏按钮保留空白。开发态 `updatePreview` 继续用于逐状态视觉验证。

## Verification

**Commands:**
- `pnpm test` -- 更新 UI 与既有前端测试通过。
- `pnpm check` -- Svelte 与 TypeScript 诊断通过。
- `git diff --check` -- 无空白与补丁格式错误。

**Manual checks (if no CLI):**
- 依次预览 current、available、downloading、downloaded、download/install/relaunch error，确认单行布局、按钮语义、无重叠和无纵向抽动。

## Suggested Review Order

**状态语义与布局**

- 状态派生保留检查前结果，并为失败提供简短恢复说明。
  [`SettingsView.svelte:31`](../../src/lib/settings/SettingsView.svelte#L31)

- 两行 Section 与单行轨道集中所有阶段动作。
  [`SettingsView.svelte:493`](../../src/lib/settings/SettingsView.svelte#L493)

- 固定轨道、可收缩标题和紧凑动作防止窄宽重叠。
  [`SettingsView.svelte:582`](../../src/lib/settings/SettingsView.svelte#L582)

**检查与文案**

- 手动检查完成后刷新最近检查时间，不覆盖其他未保存设置。
  [`SettingsView.svelte:360`](../../src/lib/settings/SettingsView.svelte#L360)

- 中英文状态文案去重并按职责拆分。
  [`catalogs.ts:226`](../../src/lib/i18n/catalogs.ts#L226)

**验证**

- 组件测试覆盖主要状态、动作、进度与错误颜色边界。
  [`SettingsView.test.ts:72`](../../src/lib/settings/SettingsView.test.ts#L72)
