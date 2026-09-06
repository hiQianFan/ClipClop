---
title: '重构 macOS 权限设置与引导信息架构'
type: 'refactor'
created: '2026-09-06'
status: 'done'
baseline_commit: 'd2407ea'
context:
  - 'DESIGN.md'
  - '_bmad-output/planning-artifacts/ux-designs/ux-ClipClop-2026-09-05/EXPERIENCE.md'
  - '_bmad-output/implementation-artifacts/spec-stabilize-unsigned-macos-auto-paste-permission.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 当前“常规”页堆叠权限说明，并复用快速入门第 3 步承担日常管理和故障恢复，导致文案冗长、页面职责混乱，单页模式仍出现无意义的“上一步/完成”。自动粘贴的状态与右侧管理动作分离，也增加了扫读成本。

**Approach:** 在 macOS 设置侧栏增加独立“权限”分类，把现有辅助功能检测、回焦复检和安全重启状态机迁入该页；常规页不再展示权限内容。首次引导仅保留简短、可跳过的授权邀请，真实粘贴失败直接打开权限分类并聚焦自动粘贴。

## Boundaries & Constraints

**Always:** 沿用现有 Svelte 5、Tauri IPC、权限状态机、设置窗口布局和设计 token；“权限”位于“快捷键”之后、分隔线之前且仅 macOS 展示；自动粘贴右侧按钮同时表达状态和动作，已就绪使用既有 success token，点击仍打开 macOS 辅助功能设置；完全磁盘访问只写“预览时验证”，不得显示总体已就绪；从失败入口进入时保留当前选中记录和来源上下文；所有状态有文字且支持键盘与读屏。

**Ask First:** 如果条件显示 macOS 专属 Tab 会破坏现有 Tabs 键盘/焦点行为，或迁移状态机必须改变 denied→granted 后的单次重启规则，则暂停确认。

**Never:** 不新增二级路由、通用路由框架或权限 store；不复制原生权限检测逻辑；不在常规页保留权限入口或详情；不让日常管理或失败恢复继续复用 onboarding step；不伪造完全磁盘访问状态；不自动操作系统设置或 TCC。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 常规设置 | 用户打开 General | 不展示权限入口或权限详情 | 维持原有布局 |
| 权限页面 | macOS 用户点击“权限” | 打开独立设置分类，无步骤器、“上一步”或“完成” | Windows 隐藏该分类 |
| 自动粘贴已就绪 | 实时检测为 ready | 右侧显示绿色“已就绪”按钮，点击仍打开辅助功能设置 | 返回后复检，撤权时更新为“去授权” |
| 自动粘贴缺权 | 实时检测为 permission_required | 右侧显示“去授权”，点击打开对应系统设置 | 打开失败原位报错；授权后维持既有单次重启闭环 |
| 文件访问 | 无可靠总体状态 | 显示“预览时验证”和“打开设置” | 仅真实 PermissionDenied 触发恢复说明 |
| 首次引导 | 到达权限步骤 | 只展示自动粘贴价值及“去授权/稍后设置” | 不展示路径、旧条目、重启工具或权限详情布局 |
| 粘贴失败恢复 | `copied_permission_required` 后点击处理 | 打开权限分类并聚焦自动粘贴行 | 保留当前历史选择 |

</frozen-after-approval>

## Code Map

- `src/lib/settings/SettingsView.svelte` -- 条件注册 macOS permissions Tab，并处理失败恢复的初始分类与焦点。
- `src/lib/settings/GeneralSettings.svelte` -- 移除权限检测与权限展示。
- `src/lib/settings/PermissionSettings.svelte` -- 新的 macOS 权限详情页，承接现有检测、系统跳转、Finder 定位和重启闭环。
- `src/lib/onboarding/OnboardingView.svelte` -- 移除权限管理模式，仅保留首次/快速入门的简短邀请。
- `src/lib/history/HistoryWorkspace.svelte` -- 设置入口和粘贴恢复统一导航到 permissions 二级页。
- `src/lib/i18n/catalogs.ts` -- 收敛中英文文案及按钮状态。
- `src/lib/settings/*.test.ts`、`src/lib/onboarding/*.test.ts`、`src/lib/history/*.test.ts` -- 迁移状态机测试并覆盖导航和回归。

## Tasks & Acceptance

**Execution:**
- [x] `SettingsView.svelte`、`GeneralSettings.svelte` -- 增加 macOS 独立权限分类并清除 General 权限内容，保持 Tabs 键盘行为和未保存设置生命周期。
- [x] `PermissionSettings.svelte` -- 从 onboarding 迁入原有辅助功能状态机，渲染状态动作按钮、文件访问行和当前 App 恢复工具。
- [x] `OnboardingView.svelte` -- 删除 `auto_paste`/`auto_paste_recovery` 管理模式和单页 footer 残留，将首次授权步骤压缩为可跳过邀请。
- [x] `HistoryWorkspace.svelte` -- 让 General、主历史和 QuickPanel 请求都进入同一 permissions 页面并聚焦自动粘贴。
- [x] `catalogs.ts` -- 删除冗长或过度技术化的可见文案，补齐权限页与状态按钮名称。
- [x] 相关测试 -- 保留 denied→granted、取消/失败/重复焦点测试，并覆盖平台导航、ready 按钮仍可管理及首次引导无管理 UI。

**Acceptance Criteria:**
- Given 用户位于权限分类，when 切换分类再返回，then 不丢失本次未保存的普通设置变更且权限重新按需检测。
- Given 自动粘贴已就绪，when 用户点击绿色“已就绪”，then 仍打开辅助功能系统设置且返回后实时复检。
- Given 用户从粘贴失败进入，when 权限页打开，then 自动粘贴行获得可感知焦点，当前历史选择保持不变。
- Given 用户首次安装但选择稍后设置，when 完成快速入门，then 不被权限详情或重启流程阻断。

## Spec Change Log

## Design Notes

权限页沿用现有 Settings Tabs，不引入应用路由。权限状态机只有一个拥有者：`PermissionSettings.svelte`；首次引导只发起系统设置动作，不尝试展示完整恢复状态。绿色仅用于已验证的辅助功能 ready 状态，完全磁盘访问继续使用中性动作。

## Verification

**Commands:**
- `pnpm check` -- Svelte/TypeScript 无错误或警告。
- `pnpm test -- --run` -- 前端权限、设置、历史和 onboarding 测试全部通过。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- 原生权限边界无回归。
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` -- Rust 格式通过。

**Manual checks (if no CLI):**
- 用 `/Applications/ClipClop.app` 检查权限分类、已就绪按钮、撤权回焦、粘贴失败恢复和首次安装稍后设置。

## Suggested Review Order

**设置导航与权限状态**

- macOS 条件注册一级权限分类并保持现有 Tabs 行为。
  [`SettingsView.svelte:20`](../../src/lib/settings/SettingsView.svelte#L20)

- 权限页集中拥有检测、回焦和单次重启状态机。
  [`PermissionSettings.svelte:42`](../../src/lib/settings/PermissionSettings.svelte#L42)

- 状态按钮沿用设置行布局并保持可操作语义。
  [`PermissionSettings.svelte:105`](../../src/lib/settings/PermissionSettings.svelte#L105)

**入口分离**

- 首次引导只保留简短且可跳过的授权邀请。
  [`OnboardingView.svelte:389`](../../src/lib/onboarding/OnboardingView.svelte#L389)

- 粘贴失败直接进入权限分类且保留历史选择。
  [`HistoryWorkspace.svelte:445`](../../src/lib/history/HistoryWorkspace.svelte#L445)

**回归验证**

- 权限状态机测试覆盖切页、复检、取消和失败边界。
  [`PermissionSettings.test.ts:15`](../../src/lib/settings/PermissionSettings.test.ts#L15)

- 深链测试覆盖已挂载设置与选择保持。
  [`HistoryWorkspace.test.ts:89`](../../src/lib/history/HistoryWorkspace.test.ts#L89)
