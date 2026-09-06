---
title: '在快速入门中展示权限配置状态'
type: 'feature'
created: '2026-09-06'
status: 'done'
route: 'one-shot'
---

# 在快速入门中展示权限配置状态

## Intent

**Problem:** macOS 快速入门只提供自动粘贴入口，缺少文件预览配置和授权完成后的状态反馈。

**Approach:** 在第 3 页用两条简洁设置行展示自动粘贴与文件预览；自动粘贴实时检测并在回焦后更新状态，文件访问保持“预览时验证”，不引入完整恢复和重启逻辑。

## Suggested Review Order

**权限状态与操作**

- 快速入门复用现有 IPC，处理检测、回焦和竞态。
  [`OnboardingView.svelte:209`](../../src/lib/onboarding/OnboardingView.svelte#L209)

- 两项权限保持简洁设置行与一致状态按钮。
  [`OnboardingView.svelte:424`](../../src/lib/onboarding/OnboardingView.svelte#L424)

**验证与文案**

- 覆盖授权、撤权、检测失败、文件设置和平台隔离。
  [`OnboardingView.test.ts:63`](../../src/lib/onboarding/OnboardingView.test.ts#L63)

- 中英文标题与状态文案保持简短。
  [`catalogs.ts:34`](../../src/lib/i18n/catalogs.ts#L34)
