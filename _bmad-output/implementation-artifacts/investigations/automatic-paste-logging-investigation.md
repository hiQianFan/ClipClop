# Investigation: 自动粘贴失败与日志体系

## Hand-off Brief

1. **What happened.** 用户报告按 Enter 后未自动粘贴；本机日志已确认多次在恢复目标窗口成功后被 macOS 拒绝事件注入。
2. **Where the case stands.** Active；已有直接失败点，尚需核对权限身份、重装前后变化及全链路日志覆盖。
3. **What's needed next.** 映射日志、源码、系统权限和版本证据，区分系统授权失效与应用实现缺陷。

## Case Info

| Field | Value |
| --- | --- |
| Ticket | N/A |
| Date opened | 2026-09-05 |
| Status | Active |
| System | macOS aarch64；ClipClop 0.8.7 |
| Evidence sources | `~/Library/Logs/com.clipclop.desktop/ClipClop.log`、源码、版本控制、系统权限状态 |

## Problem Statement

用户反馈 ClipClop 按 Enter 无法自动粘贴，即使已开启辅助功能权限；重新安装后要求读取诊断日志，并评估全应用日志设计、可观测数据和粒度。

## Evidence Inventory

| Source | Status | Notes |
| --- | --- | --- |
| ClipClop.log | Available | 10,433 bytes；2026-09-05 13:17:38–13:54:46 +0800；含多次粘贴尝试 |
| 自动粘贴源码 | Available | macOS 注入路径和工作流均存在阶段日志 |
| 当前安装包签名 | Available | `/Applications/ClipClop.app` 为 ad-hoc/linker-signed，无 TeamIdentifier、designated requirement 或 sealed resources；Gatekeeper 判定 rejected |
| 系统辅助功能授权数据库 | Missing | 当前进程无权读取用户 TCC.db；不能直接核对授权记录 |
| 版本控制 | Available | 粘贴恢复相关历史与当前未提交权限预检改动可检查 |
| 自动化测试/静态分析 | Partial | 有 Rust 单元测试，但系统 TCC 与真实事件投递无法由单元测试覆盖；本阶段未运行测试 |
| 可复现测试 | Missing | 尚未进行一次带时间标记的真实粘贴复现 |

## Investigation Backlog

| # | Path to Explore | Priority | Status | Notes |
| - | --- | --- | --- | --- |
| 1 | 重建失败时间线与权限状态变化 | High | Done | 成功者是 debug PID 38359；失败者是安装包 PID 39642，并非同一身份重启 |
| 2 | 追踪粘贴调用链和每个退出分支 | High | Done | 已映射前端、IPC、工作流、平台实现及全部结果分支 |
| 3 | 审计全应用日志覆盖、隐私、轮转和支持性 | High | In Progress | 已清点 14 个主要日志文件与关键缺口，待形成结论 |
| 4 | 核对安装包签名/路径与 TCC 授权身份 | High | Done | unified log 已确认成功和失败请求来自两个路径及两个派生签名标识 |

## Timeline of Events

| Time | Event | Source | Confidence |
| --- | --- | --- | --- |
| 2026-09-05 13:17:38 +0800 | ClipClop 0.8.7 会话启动 | ClipClop.log:1 | Confirmed |
| 2026-09-05 13:20:46–13:20:53 +0800 | 三次成功恢复目标窗口，但事件访问被拒绝 | ClipClop.log:29-42 | Confirmed |
| 2026-09-05 13:38:31 +0800 | debug 二进制 PID 38359 成功发布粘贴事件 | ClipClop.log:67-69；unified log 13:38:31 | Confirmed |
| 2026-09-05 13:38:47 +0800 | 安装版 PID 39642 启动 | ClipClop.log:75；unified log 13:38:47 | Confirmed |
| 2026-09-05 13:38:49–13:54:40 +0800 | 多次恢复目标窗口成功，但事件访问均被拒绝 | ClipClop.log:77-101 | Confirmed |

## Confirmed Findings

### Finding 1: 失败发生在 macOS 事件注入权限检查

**Evidence:** `ClipClop.log:77-80`

**Detail:** 目标窗口激活 `stable=true`，随后记录 `automatic paste event access denied`，工作流返回 `CopiedPermissionRequired`。内容已复制，但 Command-V 未注入。

### Finding 2: 同一日志窗口内至少有一次自动粘贴成功

**Evidence:** `ClipClop.log:67-69`

**Detail:** 13:38:31 的目标恢复成功，并记录 `automatic paste event posted`；说明目标捕获和事件构造并非持续性失效。

### Finding 3: 当前安装包没有可稳定绑定的正式代码签名身份

**Evidence:** `codesign -dv --verbose=4 /Applications/ClipClop.app`：`Signature=adhoc`、`TeamIdentifier=not set`、`Internal requirements=none`；`spctl` 返回 rejected。

**Detail:** Bundle plist 标识为 `com.clipclop.desktop`，但 Mach-O 的签名 Identifier 是构建派生的 `clipclop-0c64038cebd5181f`，且 Info.plist 未绑定。重建或替换二进制会改变 CDHash，macOS TCC 没有稳定的开发者签名 requirement 可用于持续识别该应用。

### Finding 4: 日志不能完整观察一次粘贴请求

**Evidence:** `src-tauri/src/workflows/paste_clip.rs:20`、`:32`；`src-tauri/src/paste/mod.rs:65`、`:85`；`src-tauri/src/paste/windows.rs:40`。

**Detail:** 并发拒绝、第一次权限预检失败、目标捕获失败都无日志；Windows 平台仅能看到最终枚举；前后端也没有一次操作的关联标识。

### Finding 5: 成功和失败来自两个不同代码身份

**Evidence:** macOS unified log 2026-09-05 13:38:31.893 与 13:38:47.533。

**Detail:** 成功会话来自 `src-tauri/target/debug/clipclop`，PID 38359、标识 `clipclop-22bceb1afd9961d1`；失败会话来自 `/Applications/ClipClop.app/Contents/MacOS/clipclop`，PID 39642、标识 `clipclop-0c64038cebd5181f`。macOS 明确将它们识别为两个访问主体。

### Finding 6: 已安装版不包含今天新增的工作流前置权限检查

**Evidence:** 安装包修改时间为 2026-09-04 22:42:09；`src-tauri/src/workflows/paste_clip.rs` 修改时间为 2026-09-05 12:13:02，且 `git diff` 显示该检查尚未提交。

**Detail:** 因此安装版日志只有平台层一次权限检查，不存在权限结果在约 90ms 内翻转的证据。

## Deduced Conclusions

### Deduction 1: 当前失败不是 Enter 未触发，也不是目标窗口恢复失败

**Based on:** Finding 1

**Reasoning:** 只有进入粘贴工作流后才会产生日志；且目标 PID 激活已稳定，退出点明确位于事件访问检查。

**Conclusion:** 当前证据把故障范围收敛到 macOS 对运行中 ClipClop 实例的事件发布授权。

### Deduction 2: debug 版本授权不能授权安装版

**Based on:** Finding 3、Finding 5

**Reasoning:** TCC 日志显示二者路径和派生标识不同；安装版又没有稳定开发者签名 requirement，系统无法将 debug 二进制的授权继承给安装包。

**Conclusion:** “辅助功能已开启”只能证明列表中某个 ClipClop 身份被允许，不能证明当前安装版身份被允许。

## Hypothesized Paths

### Hypothesis 1: 重启后的应用代码身份或运行路径与获授权实例不一致

**Status:** Confirmed

**Theory:** 13:38:31 成功后，13:38:48 新会话启动；新实例不再获得事件访问，可能由重建、重装、路径或签名身份变化导致 TCC 授权未匹配。

**Supporting indicators:** 成功与持续失败之间仅隔一次应用重启。

**Would confirm:** 对比两个实例的 bundle path、bundle ID、签名 designated requirement，并核对 TCC 辅助功能记录。

**Would refute:** 证明成功与失败实例代码身份完全相同，且失败时系统 preflight 返回允许。

**Resolution:** unified log 直接确认成功主体为仓库 debug 二进制，失败主体为安装版；两者路径和 TCC identifier 均不同。

## Missing Evidence

| Gap | Impact | How to Obtain |
| --- | --- | --- |
| 成功与失败实例的签名/路径 | 决定 TCC 是否把它们视为同一应用 | 检查当前进程、应用包和构建产物的 `codesign` 信息 |
| 当前 TCC 权限实际结果 | 区分 UI 开关与 API 判定 | 在一次受控复现前后采集 preflight/请求结果与系统设置状态 |
| 端到端关联 ID | 无法可靠关联一次 UI 操作、写剪贴板、隐藏窗口、注入及前端结果 | 审计后提出最小字段方案 |
| 系统是否真正消费 Cmd-V | `CGEventPost` 无返回值，当前“event posted”不代表目标应用已处理 | 真实应用复现或辅助 UI 自动化验证 |

## Source Code Trace

| Element | Detail |
| --- | --- |
| Error origin | `src-tauri/src/paste/macos.rs:100`，事件访问检查失败 |
| Trigger | 选择历史项后按 Enter，前端调用 `paste_clip` |
| Condition | 目标窗口恢复成功，但 macOS 拒绝 post-event access |
| Related files | `src/lib/history/HistoryWorkspace.svelte`、`src-tauri/src/commands/history.rs`、`src-tauri/src/workflows/paste_clip.rs`、`src-tauri/src/paste/mod.rs` |

完整链路：`HistoryWorkspace.svelte:523` → `history/api.ts:100` → `commands/history.rs:57` → `workflows/paste_clip.rs:11` → `paste/mod.rs:83` → `paste/macos.rs:54`。直接发布版的构建入口为 `.github/workflows/bundle.yml:75`，此前只提供 Tauri updater 的 minisign 密钥，没有 Apple 代码签名或公证凭据。

## Conclusion

**Confidence:** Medium

已确认本次未粘贴的直接原因是 macOS 拒绝当前进程发布键盘事件，而不是快捷键、剪贴板写入或焦点恢复失败。根因很可能与成功后重启产生的应用身份/TCC 授权匹配变化有关，但尚需签名、路径和权限记录证据确认。

## Recommended Next Steps

### Fix direction

1. 直接下载版固定使用同一个 `Developer ID Application` 证书、bundle ID `com.clipclop.desktop`、Hardened Runtime 和 Apple notarization；不能再发布 ad-hoc 构建。
2. 发布 CI 对 Apple 证书及公证凭据 fail closed，缺任一 secret 即停止 macOS 构建。
3. 权限仍由用户首次授予；稳定签名解决升级/重装时 TCC 身份漂移，不会绕过系统授权。

### Diagnostic

核对当前运行实例路径、签名和 TCC；随后做一次带明确时间点的失败复现。

## Reproduction Plan

记录当前实例身份与权限状态，清晰标记时间后从其他应用呼出 ClipClop、选择记录并按 Enter，再对齐日志中的一次完整操作链。

## Side Findings

- 当前日志使用 KeepOne 轮转，历史故障证据的保留能力需要专项评估。
- 全局日志级别为 Info，因此源码中的焦点与预览 debug 证据不会写入用户诊断日志（`src-tauri/src/lib.rs:65`）。
- 日志未记录剪贴板内容；但启动、设置和资源日志会暴露用户名路径、PID、快捷键及来源应用标识，应在支持包导出时脱敏。

## Follow-up: 2026-09-05

### New Evidence

- 完整日志共有 101 行，覆盖 13:17:38–13:54:46；其中 1 次事件发布成功，8 次因第二次权限检查失败而降级。
- 当前运行进程为 `/Applications/ClipClop.app/Contents/MacOS/clipclop`，PID 39642，启动于 13:38:47，与失败会话吻合。
- 日志配置为 Info、单文件 5 MiB、KeepOne；macOS 同时写文件和 stderr，Windows 仅写文件（`src-tauri/src/lib.rs:49-67`）。

### Backlog Changes

- Evidence perimeter 已映射：日志、源码、版本控制、当前签名 Available；测试 Partial；TCC 数据库与受控复现 Missing。

### Additional Findings

- 已反驳“同一 App 重启后权限由允许变为拒绝”：成功与失败来自不同二进制。
- 已反驳“第一次预检允许、第二次预检拒绝”：当前安装包早于该未提交改动，不含第一次预检。

### Updated Hypotheses

- Hypothesis 1 更新为 Confirmed：debug 与安装版身份不一致导致 TCC 授权不匹配。

### Updated Conclusion

当前未自动粘贴的直接原因是安装版没有 Post Event 权限。用户观察到的成功对应仓库 debug 二进制；macOS unified log 直接证明它与安装版是两个 TCC 主体。安装版缺少正式代码签名，使重装或升级后的权限身份稳定性更差。

### Backlog Changes #2

- `.github/workflows/bundle.yml` 已增加 Apple Developer ID 签名与 notarization 所需的五个 secrets，并在 macOS 构建前强制校验；实际签名仍等待开发者证书与账号凭据。
