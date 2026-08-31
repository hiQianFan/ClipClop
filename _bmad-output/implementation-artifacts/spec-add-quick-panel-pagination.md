---
title: 'Add Quick panel pagination'
type: 'feature'
created: '2026-08-31'
status: 'in-progress'
baseline_commit: 'be872a56f008484d00338f724304950a2572cfa1'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/DESIGN.md'
  - '{project-root}/openspec/changes/add-quick-panel-pagination/design.md'
  - '{project-root}/openspec/changes/add-quick-panel-pagination/tasks.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Quick 面板固定呈现最近一页，超过十条的历史只能跳到主面板继续浏览；当前十条记录没有填满列表区，列表与底部菜单之间还出现额外断层。

**Approach:** 复用现有 `query_history` 分页协议，在 Quick 固定十行槽位下增加 header 右侧紧凑分页栏、连续键盘浏览和请求乱序保护。十槽网格填满列表可用高度，消除额外 gap，同时不改变窗口和底部菜单位置。

## Boundaries & Constraints

**Always:** 正常工作区每页十条；十槽网格填满列表可用高度且不足十条保留空槽；分页栏固定在 header 右侧；重新打开和历史变化回到第一页；打开完整历史携带当前选择；鼠标、PageUp/PageDown 与跨页 Up/Down 均可达；加载失败保留原页；所有可见文字走 i18n；复用现有 API、类型和原生按钮。

**Ask First:** 改 Quick 窗口尺寸、改变底部菜单、占用 Left/Right、修改 Rust/数据库、引入依赖，或让 Quick 复用会加载 detail 的 `HistorySession`。

**Never:** 复制主面板拖拽 scrubber、加入 haptic/动画状态机、滚动代替分页、隐藏已查询记录、因最后一页不足而压缩窗口或拉伸行高、覆盖当前工作区的发布流程与非 Quick 修改。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| 正常分页 | 23 条、标准窗口 | 3 页；前两页各十条，末页三条加七个空槽 | N/A |
| 键盘跨页 | 第一页末项按 Down / 第二页首项按 Up | 下一页首项 / 上一页末项 | 请求期间忽略重复导航 |
| 快速翻页 | 多个请求乱序返回 | 只提交最后一次请求 | 丢弃过期响应 |
| 请求失败 | 当前页加载下一页失败 | 保留当前页、页码、选择并显示 inline error | 可重试 |
| 数据变化 | Quick 重开或 `history_changed` | 刷新第一页并选中最新项 | 空历史显示既有空状态 |

</frozen-after-approval>

## Code Map

- `src/lib/history/api.ts` -- 为现有历史查询增加可选 page size，主面板默认仍为十。
- `src/lib/history/quick-keyboard.ts` -- Quick 的纯键盘路由与跨页意图。
- `src/lib/history/QuickPanel.svelte` -- 分页状态、请求保护、固定槽位和分页栏 owner。
- `src/lib/i18n/index.svelte.ts` -- 分页按钮和页码的中英文文案。

## Tasks & Acceptance

**Execution:**
- [ ] 先扩展 API/键盘测试，覆盖 page size、PageUp/PageDown、上下键边界。
- [ ] 将 Quick 数据状态改为 `HistoryPage`，加入最后请求胜出、失败保留和回第一页语义。
- [ ] 在 header 右侧加入固定分页栏，以十槽网格渲染内容/空槽并消除列表底部额外 gap。
- [ ] 补组件测试并运行完整前端/OpenSpec 门禁。

**Acceptance Criteria:**
- Given 标准 Quick 面板，when 任一页面少于十条，then 十行区域、分页栏和底部菜单位置不变，空槽不可聚焦。
- Given 多页历史，when 使用按钮、PageUp/PageDown 或在边界继续按 Up/Down，then 目标页与选择正确且数字键仍作用于当前页。
- Given 翻页失败或请求乱序，when 响应结束，then 已提交页面不闪空、不被过期结果覆盖。

## Spec Change Log

## Design Notes

分页栏位于 header 右侧；单页显示低权重 `1/1` 与禁用箭头。列表使用十槽网格完整填充自身可用高度，最后一页的空槽保留几何但不可聚焦，红框中的额外断层不再存在。Quick 保留轻量摘要查询，不引入完整 HistorySession。

## Verification

**Commands:**
- `pnpm check && pnpm test && pnpm build && git diff --check` -- 全部通过且无 CSS syntax/minify warning。
- `openspec validate add-quick-panel-pagination --strict` -- 通过。

**Manual checks:**
- macOS/Windows：1、10、11、23 条记录；按钮与键盘翻页；末页空槽；浅深主题；隐藏重开回第一页。
