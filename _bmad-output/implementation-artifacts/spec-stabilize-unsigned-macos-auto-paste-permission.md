---
title: '稳定未正式签名 macOS 安装版的权限授权旅程'
type: 'bugfix'
created: '2026-09-05'
status: 'done'
baseline_commit: '5f3658b'
context:
  - 'docs/architecture.zh-CN.md'
  - '_bmad-output/implementation-artifacts/investigations/automatic-paste-logging-investigation.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 未购买 Apple Developer Program 时，ClipClop 安装包只能使用 ad-hoc 身份；首次安装、覆盖更新或重新构建后，macOS 可能不认旧辅助功能或完全磁盘访问条目。目前用户无法判断授权对象是否为当前安装版，也缺少从功能失败回到正常使用的闭环。

**Approach:** 在 Settings → General 建立“权限与系统访问”分组，并从粘贴/预览真实失败处提供原位恢复入口。辅助功能形成“实时检测 → 登记当前进程 → 打开设置 → 回焦复检 → 预告并可取消的单次重启”闭环；完全磁盘访问不伪造全局状态，只根据具体文件读取结果反馈。

## Boundaries & Constraints

**Always:** 仅在 macOS 启用该流程；辅助功能始终检查当前运行进程，不缓存结果；完全磁盘访问只按具体文件的 `PermissionDenied` 判断；不读取或修改 TCC；不记录剪贴板或文件内容；使用现有 Svelte、Tauri process plugin、onboarding 页面和设计 token；明确区分 `/Applications/ClipClop.app` 与开发构建；只有本次引导观察到 denied→granted 才预告并自动重启一次。

**Ask First:** 实现中若无法可靠解析当前 `.app` bundle、无法区分文件 `PermissionDenied` 与其他预览错误，或需要新增第三项系统权限，必须暂停确认。

**Never:** 不调用 `tccutil reset`，不删除用户权限记录，不伪造签名，不尝试自动点击系统设置，不因缺权限而丢弃剪贴板写入，不让后台轮询常驻运行，不把 debug 与安装版描述成同一授权对象。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 首次引导 | 新用户 | 解释两项能力与可选性；可配置自动粘贴或稍后处理 | 不要求一次性全部授权 |
| 辅助功能授权 | 当前实例 denied | 请求系统登记当前进程并打开辅助功能设置；页面显示等待状态 | 打开失败时留在引导页并显示本地化错误 |
| 授权完成 | 引导已启动，窗口重新获得焦点，当前实例 granted | 记录状态转换并自动 relaunch 一次 | relaunch 失败时显示“重启 ClipClop”按钮 |
| 未完成授权 | 返回 App 后仍 denied | 保留引导与重试按钮，不重启、不循环弹窗 | 提供 Finder 定位和手动添加说明 |
| 普通粘贴 | 当前实例 granted | 写剪贴板、隐藏面板、恢复目标并注入 Cmd-V | 继续沿用既有结果枚举 |
| 权限失效 | 粘贴时 denied | 面板不隐藏；内容写入剪贴板；先显示带“处理权限”的原位提示，不强制跳转 | 用户可忽略并手动 Cmd-V |
| 文件访问失败 | 具体文件读取返回 `PermissionDenied` | 保留选择并显示完全磁盘访问恢复入口 | 其他预览错误不得误报为权限问题 |
| 更新后身份改变 | 新安装版 denied、旧条目仍存在 | 明示移除旧条目并重新添加当前 `/Applications/ClipClop.app` | 不声称旧授权可迁移 |
| 非 macOS | Windows/其他平台 | 不展示 macOS 权限引导或重启逻辑 | 维持现有平台行为 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/paste/macos.rs` -- 调用 `CGPreflightPostEventAccess` 与 `CGRequestPostEventAccess`。
- `src-tauri/src/paste/mod.rs` -- 跨平台权限能力边界。
- `src-tauri/src/commands/onboarding.rs` -- 暴露权限状态、打开设置及 Finder 定位命令。
- `src-tauri/src/workflows/paste_clip.rs` -- 粘贴前实时预检并保留 clipboard-only 降级。
- `src/lib/onboarding/api.ts` -- 权限状态与原生命令 IPC 类型。
- `src/lib/onboarding/OnboardingView.svelte` -- 权限状态机、回焦复检、重启和手动恢复 UI。
- `src/lib/history/HistoryWorkspace.svelte` -- 从粘贴失败进入权限引导。
- `src/lib/settings/GeneralSettings.svelte` -- “权限与系统访问”分组与两项独立管理入口。
- `src-tauri/src/assets/`、`src-tauri/src/preview/` -- 仅将真实文件访问拒绝映射为权限恢复结果。
- `src/lib/i18n/catalogs.ts` -- 中英文状态、操作与限制说明。
- `src-tauri/src/lib.rs` -- 注册新增命令。

## Tasks & Acceptance

**Execution:**
- [x] Rust 权限 API -- 返回 `ready`/`permission_required`/`unsupported`，并记录请求、复检与身份上下文（版本、bundle ID、开发/安装路径类别，不记录用户名完整路径）。
- [x] 权限恢复入口 -- 请求当前实例、打开系统设置，并提供 Finder 定位当前 `.app`；无法定位 bundle 时隐藏该操作。
- [x] 前端状态机 -- 仅在引导激活期间监听窗口回焦并复检；成功转换后自动 relaunch 一次，失败可手动重试。
- [x] 粘贴失败入口 -- 保持面板可见并提供明确的“处理权限”操作，不把普通错误文本做成不可操作死路。
- [x] 文件预览入口 -- 仅在真实 `PermissionDenied` 时提供完全磁盘访问引导，其他错误保持原诊断。
- [x] 设置分组 -- 在 General 内集中展示两项能力，不新增侧栏分类或虚假总体完成度。
- [x] 测试 -- 覆盖 denied、granted、回焦未授权、授权后单次重启、重启失败、非 macOS与重复焦点事件。

**Acceptance Criteria:**
- Given 用户授权的是旧构建，when 当前安装版预检失败，then UI 不显示“已授权”，并引导添加当前实例。
- Given 内容已复制但无权限，when 用户忽略引导，then 可正常手动粘贴且历史数据不丢失。
- Given 本次引导观察到 denied→granted，when App 回到前台，then 最多自动重启一次且日志可解释该决定。
- Given 权限原本已 granted，when 正常启动或粘贴，then 不弹权限页、不重启且无常驻轮询。

## Spec Change Log

## Design Notes

实时 `CGPreflightPostEventAccess` 很轻量，继续在每次实际粘贴边界调用；只有权限引导页在窗口回焦时额外调用一次。不要做定时轮询或持久化权限布尔值，因为 TCC 可被用户随时撤销，缓存反而制造旧状态。

## Verification

**Commands:**
- `pnpm test -- --run` -- 前端测试全部通过。
- `pnpm check` -- Svelte/TypeScript 无错误或警告。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 测试全部通过。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- Rust 格式通过。

**Manual checks (if no CLI):**
- 使用 `/Applications/ClipClop.app` 分别验证首次拒绝、授权返回、自动重启、更新后旧条目和手动 Cmd-V 兜底。

## Suggested Review Order

**权限状态与恢复闭环**

- 从实时检测、回焦复检到单次重启的核心状态机。
  [`OnboardingView.svelte:85`](../../src/lib/onboarding/OnboardingView.svelte#L85)

- 原生命令返回当前实例权限与安全的安装位置身份。
  [`onboarding.rs:76`](../../src-tauri/src/commands/onboarding.rs#L76)

- 纯函数约束只有本次授权转换才能触发重启。
  [`api.ts:25`](../../src/lib/onboarding/api.ts#L25)

**真实失败入口**

- 主窗口统一承接设置页和粘贴失败的权限引导。
  [`HistoryWorkspace.svelte:402`](../../src/lib/history/HistoryWorkspace.svelte#L402)

- 快捷面板复用既有窗口消息进入同一恢复闭环。
  [`history.rs:91`](../../src-tauri/src/commands/history.rs#L91)

- 设置页只展示两项独立能力，不伪造总体授权状态。
  [`GeneralSettings.svelte:63`](../../src/lib/settings/GeneralSettings.svelte#L63)

**验证**

- 组件测试覆盖授权、取消、失败及重复焦点边界。
  [`OnboardingPermission.test.ts:1`](../../src/lib/onboarding/OnboardingPermission.test.ts#L1)

- 快捷面板测试确保权限错误不会残留或绕过引导。
  [`QuickPanel.test.ts:1`](../../src/lib/history/QuickPanel.test.ts#L1)
