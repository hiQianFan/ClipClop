---
title: 'Persist update check time display across restarts'
type: 'bugfix'
created: '2026-09-03'
status: 'in-review'
baseline_commit: '6f96ddacc06ff3acb21027398dbfe1cd179b7a7d'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 软件更新页把持久化的 `last_update_check` 绑定到当前进程内的 `current` 状态。应用重启或 WebView 重建后状态回到 `idle`，即使数据库已有检查时间，界面仍错误显示“尚未检查更新”。

**Approach:** 让状态轨道在无活动更新结果时直接使用已加载的检查时间；仅在本次检查确实确认最新版时显示“已是最新版本”。首次安装且从未检查时保留明确的首次检查占位语义。

## Boundaries & Constraints

**Always:** 使用设置中现有的 `last_update_check`；重启、关闭重开窗口和升级后只要该值存在就显示格式化日期时间；保留检查中、可用更新、下载、安装和错误状态的现有优先级；中英文行为一致；补回归测试。

**Ask First:** 改变自动检查的 24 小时间隔或 15 秒启动延迟；改变“检查尝试”时间戳在网络请求前记录的语义。

**Never:** 不新增数据库字段或第二套缓存；不把历史检查时间伪装成本次已确认最新版；不在从未成功记录检查时间时伪造日期；不改变更新下载、安装或发布逻辑。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 重启后恢复 | `phase=idle`，有 `last_update_check`，无可用更新 | 状态轨道显示“上次检查：日期时间”，不显示“尚未检查更新” | 无 |
| 首次启动 | `phase=idle`，无 `last_update_check`，无可用更新 | 显示首次检查占位文案，不伪造时间 | 自动检查仍按既有计划执行 |
| 本次确认最新版 | `phase=current`，有检查时间 | 显示“已是最新版本”和检查时间 | 无 |
| 检查失败 | `phase=error`，历史时间存在 | 保留失败标题和恢复说明 | 不用历史结果掩盖错误 |
| 有可用更新 | `phase=idle`，缓存中有更新 | 显示新版本及现有动作 | 历史时间不覆盖更新提示 |

</frozen-after-approval>

## Code Map

- `src/lib/settings/UpdateSettings.svelte` -- 派生状态轨道的主文案和辅助检查时间。
- `src/lib/settings/SettingsView.test.ts` -- 设置页更新状态的 DOM 回归测试。
- `src/lib/i18n/catalogs.ts` -- 首次检查占位文案的中英文来源；仅在需要调整现有文案时修改。
- `src-tauri/src/settings/service.rs` -- 已有 SQLite 时间戳持久化边界，仅验证、不修改。

## Tasks & Acceptance

**Execution:**
- [x] `src/lib/settings/UpdateSettings.svelte` -- 在 `idle` 且没有可用更新时优先呈现持久化检查时间，同时保持活动状态优先级。
- [x] `src/lib/settings/SettingsView.test.ts` -- 增加重启等价状态及首次启动回归覆盖，并保留现有 current/error/available 测试。
- [x] `src/lib/i18n/catalogs.ts` -- 如需替换“尚未检查更新”，以准确、不伪造检查结果的首次状态文案同步中英文。

**Acceptance Criteria:**
- Given 数据库已有上次检查时间，when 应用重启后打开更新页，then 用户无需再次检查即可看到该日期时间。
- Given 更新检查、下载或安装正在进行，when 状态轨道渲染，then 活动状态和恢复动作不被历史时间覆盖。
- Given 用户从未完成过检查时间记录，when 首次打开更新页，then 页面不声称存在历史检查日期或已确认最新版。

## Spec Change Log

## Verification

**Commands:**
- `pnpm test -- --run src/lib/settings/SettingsView.test.ts` -- 更新状态回归测试全部通过。
- `pnpm check` -- Svelte 与 TypeScript 诊断通过。
- `git diff --check` -- 补丁无空白错误。
