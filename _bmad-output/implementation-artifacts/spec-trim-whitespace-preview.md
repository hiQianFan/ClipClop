---
title: 'Match text preview to whitespace setting'
type: 'bugfix'
created: '2026-08-19'
status: 'done'
route: 'one-shot'
---

# Match text preview to whitespace setting

## Intent

**Problem:** 开启去除首尾空白后，Enter 粘贴的文本已正确裁剪，但右侧详情仍展示原始首尾空白。

**Approach:** 保留历史原始内容，在历史工作区同步现有设置，并让已加载的文本详情按同一开关裁剪显示。

## Suggested Review Order

**展示语义**

- 共享展示函数保持关闭原样、开启裁剪。
  [`presentation.ts:74`](../../src/lib/history/presentation.ts#L74)

- 右侧文本详情使用与粘贴一致的设置。
  [`ClipPreview.svelte:58`](../../src/lib/history/ClipPreview.svelte#L58)

**设置传递与验证**

- 关闭设置页后同步并传入详情组件。
  [`HistoryWorkspace.svelte:104`](../../src/lib/history/HistoryWorkspace.svelte#L104)

- 最小测试锁定开关两种显示结果。
  [`presentation.test.ts:30`](../../src/lib/history/presentation.test.ts#L30)
