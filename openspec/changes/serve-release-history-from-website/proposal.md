## Why

App 的版本历史、官网 changelog 与官网首页 latest release 目前在用户侧直连 GitHub API。匿名请求共享出口 IP 的额度，真实排查已出现 HTTP 403、`X-RateLimit-Remaining: 0`；客户端分页与本地缓存只能降低频率，不能消除共享代理出口限流。公开 GitHub release 数据是低频变动内容，应统一生成静态数据文件，由官网域名分发。

## What Changes

- 在主仓库 CI 中使用现有 `GITHUB_TOKEN` 集中获取正式 GitHub Releases，生成单个静态 `releases.json` 并上传现有 `clipclop-releases` R2 bucket。
- website 通过 `https://clipclop.io/releases.json` 分发该文件，只读 R2，不在用户请求期间访问 GitHub。
- App 的版本历史、官网 changelog、官网首页 latest release 共用上述 releases 数据源；App 与官网仍可在界面上按 10 条逐步展示，但不再进行网络分页。
- website header 不再动态请求 GitHub stars；保留 GitHub 链接，必要时使用静态文案。
- 发布流程成功公开正式 release 后显式同步；提供手动同步、每 6 小时补偿同步，覆盖历史回填及已发布说明的编辑、撤回。
- 同步失败保留上一次可用 JSON。
- App 调整响应验证、错误提示、CSP；website 配置 CORS、缓存与精确路由。

## Capabilities

### New Capabilities

- `releases-distribution`: 发布记录的集中同步、静态分发及 App/官网消费契约。

### Modified Capabilities

- 无已落地的 `openspec/specs` 能力需要修改；本变更以新增能力定义跨仓库契约。

## Scope and Non-Goals

- 迁移用户侧 GitHub Releases 动态请求：版本历史、更新说明和官网首页 stable version 文案。
- `/latest.json`、`/download/*`、安装包路径、更新检查、签名校验、下载与安装保持原有契约。
- 不新增常驻抓取服务、数据库、用户账号、App Token 设置或客户端内置凭据。
- 不重做 changelog 样式或引入新的 Markdown 编辑器。
- 不把 GitHub 请求变成实时网站代理；网站流量不得转换为 GitHub 请求量。
- 不为 GitHub stars 单独维护数据链路；header stars 数字从本次范围移除。

## Impact

- 主仓库：`.github/workflows/bundle.yml`、新增可复用同步 workflow 与生成脚本、`src/lib/updater/api.ts`、`src/lib/settings/ReleaseNotes.svelte`、i18n、`src-tauri/tauri.conf.json`、测试与 `docs/distribution.md`。
- 网站仓库 `/Users/qianfan/Desktop/Code/clipclop.io`：`worker.js`、`worker.test.js`、`wrangler.toml`、`src/features/changelog/changelog.ts`、`src/features/home/home.ts`、`src/components/SiteHeader.astro`、分发文档与对应 OpenSpec 变更。
- 复用已有 R2、Worker 和发布 secrets；不新增跨仓库 dispatch 凭据。公共 JSON 与 HTML 不得携带 GitHub 或 Cloudflare 凭据。
- 既有已发布 App 不会自动切换来源，需要发布新版；发布说明同步失败不撤销已经成功发布的 App。

## Implementation Gate

用户已确认本提案并要求推进。当前实施范围是代码、CI、Worker 路由、客户端迁移和文档；尚未执行生产 R2 回填、website 部署或新版 App 发布。
