---
title: 'ClipClop 无缝自动更新'
type: 'feature'
created: '2026-07-19'
status: 'in-progress'
baseline_commit: '504bc81'
context:
  - 'docs/architecture.md'
  - 'docs/distribution.md'
  - 'outputs/research/technical-desktop-auto-update-research-2026-07-14.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** ClipClop 目前只能构建并上传临时 Actions artifacts，已安装用户无法发现、验证并无缝安装新版本；macOS/Windows 的正式个人分发产物和更新链路也尚未收敛。

**Approach:** 使用 Tauri 2 官方 Updater + Process 插件，以公开 GitHub Releases 的 `latest.json` 为唯一更新源；macOS 发布 Universal DMG、Windows 发布 x64 NSIS EXE，应用在设置页支持手动检查、下载进度和确认安装，并在启动后进行低频非打扰检查。

## Boundaries & Constraints

**Always:** 只有通过内置公钥验证的更新才能安装；私钥不得进入仓库、日志、普通 CI 或聊天输出；Release 默认创建为 Draft；更新失败不能影响剪贴板捕获、数据库或应用启动；自动检查最多每 24 小时一次且不在每次唤起 Quick Panel 时触发；保留 GitHub Release 手动下载作为回退；复用现有灰阶设置 UI 和本地 SQLite 设置。

**Ask First:** 将本地 updater 私钥上传到 GitHub `production-release` Environment Secret 前；创建或公开 GitHub Release 前；发现必须改变数据库 schema、GitHub 仓库可见性或引入 Apple/Windows 付费代码签名时。

**Never:** 自建更新服务器、接入应用商店、MSIX/MSI、Sparkle/WinSparkle、多发布通道、强制更新、静默无提示安装、自动降级、遥测、上传设备或剪贴板数据；不把 DMG 当作 macOS updater artifact，也不为 Intel/ARM 分发两个独立 DMG。

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| 已是最新 | endpoint 返回相同/更低 SemVer | 设置页显示已是最新，不下载 | 保持当前版本 |
| 发现更新 | 返回更高版本与有效签名元数据 | 展示版本、说明和安装入口 | 用户可稍后处理 |
| 下载更新 | 用户确认安装 | 展示真实下载进度，验证后安装并重启/退出安装 | 可重试或打开 Release |
| 网络失败 | GitHub 不可达/超时 | 不阻塞启动，设置页显示简洁错误 | 24 小时节流不锁死手动检查 |
| 签名失败 | artifact 被替换或签名不匹配 | 硬停止安装 | 显示安全错误与 Release 回退 |
| 开发/Web 环境 | 不在 Tauri runtime 或 debug | 页面仍可加载，更新控件明确不可用 | 不产生未处理异常 |

</frozen-after-approval>

## Code Map

- `src-tauri/Cargo.toml`、`package.json` -- Updater/Process 原生与前端依赖。
- `src-tauri/src/lib.rs`、`src-tauri/capabilities/default.json` -- 插件注册和最小权限。
- `src-tauri/tauri.conf.json` -- 公钥、GitHub endpoint、updater artifacts 与 Windows passive 模式。
- `src/lib/updater/api.ts` -- 单一更新状态机、检查节流、下载/安装与 Release 回退。
- `src/routes/settings/+page.svelte` -- 当前版本、检查状态、Release Notes、进度和确认操作。
- `src-tauri/src/commands/settings.rs`、`src/lib/settings/api.ts` -- 自动检查偏好与上次检查时间持久化。
- `.github/workflows/bundle.yml` -- Universal DMG、Windows NSIS、签名、`latest.json` 和 Draft Release。
- `README.md`、`docs/distribution.md`、`docs/privacy.md`、`docs/release-checklist.md` -- 用户升级、安全边界和发布操作。

## Tasks & Acceptance

**Execution:**
- [ ] 创建 `hiQianFan/ClipClop` 公开仓库并配置 `origin`，不发布 Release。
- [x] 安装并配置 Updater/Process、公钥、endpoint 与 capability；不修改维护者本地私钥文件或权限。
- [x] 增加更新状态模块和设置持久化，保证并发检查去重与 24 小时节流。
- [x] 扩展设置页更新区，覆盖矩阵中的正常、下载、完成与失败状态。
- [x] 将 bundle workflow 收敛为 macOS Universal DMG + Windows x64 NSIS，并由 `tauri-action` 创建 Draft Release/updater JSON。
- [x] 补充可自动化的状态逻辑测试、配置验证和更新文档。

**Acceptance Criteria:**
- Given 已安装旧版本且公开 Release 存在有效新版本，when 用户检查并确认更新，then 对应平台下载、验签、安装并进入新版本且历史/设置保留。
- Given 普通 PR 或质量 workflow，when CI 运行，then 无法访问 updater 私钥且不会创建或修改 Release。
- Given `vX.Y.Z` 触发受保护发布，when 双平台构建成功，then Draft Release 只提供一个 Universal DMG、一个 Windows x64 setup EXE及所需 updater 技术文件。
- Given 网络或签名异常，when 检查/安装失败，then 应用继续正常工作并提供可理解的恢复入口。

## Spec Change Log

## Design Notes

首版保留用户确认步骤：启动后只检查，不自动下载；设置页是唯一完整更新界面。Windows 使用 passive NSIS 安装，macOS ARM/Intel 运行时目标共同指向 Universal updater archive。GitHub `production-release` Environment 是私钥唯一线上使用边界。

运行中的 ClipClop 只内置公钥，不读取或存储私钥。`~/.tauri/clipclop.key` 只是维护者本地备份位置；Release CI 通过 GitHub Environment Secret 注入私钥内容与密码。`.tauri` 是 Tauri 文档示例采用的惯例路径，不是运行时强制目录，移动文件不会影响已安装应用，只需同步更新维护记录和 CI Secret。

## Verification

**Commands:**
- `pnpm check && pnpm build` -- Svelte/TypeScript 零错误并完成生产构建。
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml && cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings && cargo test --manifest-path src-tauri/Cargo.toml` -- Rust 全部通过。
- `pnpm tauri build --target universal-apple-darwin --bundles dmg`（带测试签名环境）-- 生成 DMG、`.app.tar.gz` 与 `.sig`。
- `gh workflow view Bundle` 与 Actions 验证 -- Windows job 生成 NSIS EXE，Release 保持 Draft，`latest.json` 指向正确 tag/平台。

**Manual checks (if no CLI):**
- 在 macOS 与 Windows 上执行前一版本到候选版本的真实升级，确认进度、重启、版本号、SQLite 历史与设置。
