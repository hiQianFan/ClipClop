---
title: 'Deduplicate update check status'
type: 'bugfix'
created: '2026-08-19'
status: 'done'
route: 'one-shot'
---

# Deduplicate update check status

## Intent

**Problem:** 检查更新时，状态区和检查按钮同时显示“正在检查”，形成重复反馈。

**Approach:** 让右侧按钮独占可见的检查中状态，左侧仅显示完成结果或错误；检查中的完整文本保留为原子 live region，维持辅助技术反馈。

## Suggested Review Order

- 检查中仅向视觉用户展示按钮状态，同时保留读屏播报。
  [`SettingsView.svelte:445`](../../src/lib/settings/SettingsView.svelte#L445)

- 按钮继续承担加载图标、禁用和 busy 状态。
  [`SettingsView.svelte:449`](../../src/lib/settings/SettingsView.svelte#L449)
