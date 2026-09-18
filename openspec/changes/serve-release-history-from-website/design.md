## Context

已检查的事实源：

- 主仓库 `docs/distribution.md`、`.github/workflows/bundle.yml`：主仓库已有 R2 写权限，先上传版本资产和 manifests，最后通过 `gh release edit --draft=false` 公开发布。
- website 的 `worker.js`、`wrangler.toml`、`docs/distribution.md`：已有 `RELEASES` 只读 binding，网站统一持有域名、Worker 与下载路由；生产部署由 website 自己的 workflow 管理。
- website `src/features/changelog/changelog.ts`：浏览器请求 GitHub 前 10 条，按语言抽取 Markdown，存在本地旧数据回退。
- website `src/features/home/home.ts`：浏览器请求 GitHub latest release，用于首页 stable version 文案。
- website `src/components/SiteHeader.astro`：浏览器请求 GitHub repo API，用于 stars 展示，6 小时本地缓存；本变更移除该动态数字。
- App `src/lib/updater/api.ts`：GitHub 每页 10 条，缓存成功分页 promise；`ReleaseNotes.svelte` 负责追加和重试。`notesHtml` 来自 GitHub 渲染，而非自己把 Markdown 拼成 HTML。
- App 的正式 updater endpoint 已经是 `https://clipclop.io/latest.json`，但 WebView `connect-src` 仍只列 GitHub；原生 updater 与 WebView 的请求权限不能混为一谈。

## Decision 1: 复用发布资产的生产与分发分工

```text
GitHub 正式 Releases
    ↓ CI 使用 GITHUB_TOKEN，有界分页抓取
主仓库同步 workflow → 校验、净化、生成 releases.json → R2
    ↓ website Worker 只读分发
clipclop.io/releases.json
    ├── App 设置 / 版本历史
    ├── 官网 /zh/changelog 与 /en/changelog
    └── 官网首页 stable version 文案
```

website header 不再显示动态 stars 数字，因此不需要 repo summary 数据源。保留 GitHub 链接即可覆盖导航需求。

“website 统一提供数据”不要求把抓取凭据也迁移到 website。由现有 R2 写入方生成静态文件，website 保持只读，是当前仓库结构下最小改动。网站无需因每次发布而重新构建或部署。

备选方案：website CI 生成并提交静态 assets 会引入发布通知/定时重建与网站部署耦合；Worker 实时缓存代理需要运行时 Token、刷新锁和故障回退；两者本次不选。

## Decision 2: v1 单文件 releases 契约

公开文件 `GET /releases.json` 包含完整正式 release 列表。该文件也作为官网首页 latest release 的来源，避免再请求 GitHub `/releases/latest`。App 和官网 changelog 可以在 UI 上按 10 条逐步显示；这是客户端切片，不是网络分页。示例字段：

```json
{
  "schemaVersion": 1,
  "generatedAt": "2026-09-17T00:00:00Z",
  "contentHash": "<64-character-lowercase-sha256>",
  "total": 21,
  "releases": [{
    "version": "v0.11.1",
    "publishedAt": "2026-09-11T02:08:00Z",
    "name": "ClipClop v0.11.1",
    "notes": "## 中文\n…\n## English\n…",
    "notesHtml": "<h2>中文</h2>…",
    "url": "https://github.com/hiQianFan/ClipClop/releases/tag/v0.11.1",
    "isLatest": true
  }]
}
```

- 空集合返回 `total: 0` 和空数组。
- 空集合返回 `total: 0` 和空数组。
- `contentHash` 为归一化发布数据的 SHA-256，不把生成时间计入 hash；内容未变化时不重复写入。
- 入口对应 R2 key `releases.json`。先上传并校验临时对象，成功后再替换正式文件；R2 没有多对象事务，但单文件替换足够覆盖本场景。
- 不保留 `snapshots/<hash>/page-N.json`。当前历史规模很小，完整 JSON 由 CDN 分发，请求成本低；网络分页带来的对象数量、路径校验、旧快照保留和回滚复杂度不值得。
- 如果未来 release notes 体积明显增长，再定义新的静态文件或目录版本；当前不提前背这个复杂度。

## Decision 3: 内容范围、同步与失败语义

- 使用认证 GitHub Releases API，CI 每页最多 100 条并跟随 GitHub 分页。仅收录非 draft、非 prerelease 的公开正式版本；按 `published_at` 降序排列，以 release ID 作为同时间的稳定次序；去重后最新一条标记 `isLatest`。
- 首次迁移回填全部正式历史，后续重建完整 JSON；这是 CI 的低频全量操作，客户端只请求官网 JSON。设置最多 100 个 GitHub 页面、单请求 30 秒、作业 15 分钟的上限；触限时失败，不发布截断结果。
- 抓取使用支持 Markdown 与 HTML 的 full media type；保留完整双语 Markdown，官网继续抽取当前语言，App 保留现有双语显示。
- 缺失关键字段、非法日期、重复冲突、部分请求失败均不得替换 index。网络/限流只做有界重试，尊重 Retry-After / reset；等待超出作业预算时退出，保留旧 JSON。
- 在 `bundle.yml` 公开 release 成功后显式调用同一个同步实现；不单独依赖 `release.published`，因为由 `GITHUB_TOKEN` 引发的事件不能作为另一 workflow 一定运行的前提。
- 同步提供 `workflow_dispatch` 与每 6 小时 schedule；编辑/删除 release 最迟在下一次成功同步反映，也可手动立即同步。同步作业使用独立的固定并发组，不允许多个写入者交错发布。
- 所有触发入口复用一个 reusable workflow；直接使用主仓库 `GITHUB_TOKEN` 的 `contents: read` 与现有 production-release 的 R2 凭据。定时任务若受 environment 审批策略影响，实施时验证并采用已有允许的 CI 配置，不绕过审批。
- 同步失败在 Actions 明确失败并记录阶段、HTTP 状态和生成版本，不泄露凭据、不覆盖旧 index；已经公开的 release、下载与 updater manifests 保持可用。运营可手动重跑同步。

## Decision 4: Website 分发、缓存与信任边界

- website 增加精确 `/releases.json` 路由和 `run_worker_first` 配置；拒绝路径穿越、任意 R2 key 与外部转发。
- 只支持 GET、HEAD、OPTIONS；合法不存在对象返回 JSON 404，R2 读取异常返回 JSON 503，其他方法 405。API 不进行语言重定向，不回退为 Astro HTML 页面。
- 公共只读数据统一设置 `Access-Control-Allow-Origin: *`（不带 credentials）、允许 GET/HEAD/OPTIONS；成功、错误与 OPTIONS 均有 CORS。返回 JSON MIME、`nosniff` 和 ETag。
- `releases.json` 使用 `public, max-age=300`；错误 `no-store`。明确配置和验证 Cloudflare 对该文件的边缘缓存，不假定 R2 binding 返回响应就自动进入 CDN；HEAD/条件请求不误缓存空 GET 正文。
- 手动刷新重验证 `releases.json`；普通请求在最多 5 分钟内看到新内容。
- HTML 依然是不可信边界：发布端使用 GitHub 渲染后的 `body_html`，生成前拒绝脚本、事件属性和危险 URL；App 消费前再次拒绝脚本、iframe、表单、远程图片、事件属性和危险 URL，失败时降级为 Markdown 文本。
- App 验证 schemaVersion、字段类型、日期以及发布链接的 GitHub 仓库归属。未知 schema 或无效响应进入可恢复错误状态，不更新成功缓存。

## Decision 5: 消费方迁移与 UX

- App 首次请求完整 `releases.json`，本地按 10 条递增展示，保留现有请求去重、旧响应失效保护和会话内成功缓存。手动刷新重读文件；不自动回退 GitHub。
- WebView CSP 增加 `https://clipclop.io`，审计其他调用后再决定是否移除 GitHub；不得因本次迁移扩大为任意 HTTPS 域名。
- 首次获取失败显示居中的“版本历史暂时无法加载 / 重试”，只在请求进行中显示骨架屏。已有内容时刷新或追加失败保留内容，显示局部重试；不再向终端暴露 GitHub 额度提示作为正常站点错误。
- 官网 changelog 消费同一 index，保留语言抽取、原有本地旧数据回退，缓存 key 区分 v1 新结构；界面按需显示更多旧版本，不发额外网络请求。
- 官网首页 stable version 文案从 `releases.json` 的第一条 release 读取；缺失时保留静态默认文案，不触发 GitHub 请求。
- 官网 header 移除动态 stars 请求，保留 GitHub 链接，不触发 GitHub API。
- 每个客户端请求有 15 秒超时；超时或响应无效不得污染成功缓存。网站没有可用 JSON 时返回明确 404/503，不伪造空历史。

## Rollout and Rollback

1. 用户确认本提案；在 website 仓库补齐对应变更记录和契约引用。
2. 完成生成器和 dry-run 测试，准备真实全历史样本；先回填 R2，再部署 website 路由。部署必须经过网站现有生产工作流。
3. 验证公网 GET/HEAD/OPTIONS、CORS、缓存、HTML 安全及故障响应后，才将官网 changelog 和 App 切到新来源。
4. 发布新版 App；验证 macOS、Windows 的正式 WebView，而非只在 Vite 浏览器验证 CSP/CORS。
5. 数据回滚：保存并恢复前一份 index。路由实现回滚必须保留 v1 API 兼容，不得回滚到完全不支持新 API 的 Worker。
6. 旧 App 仍可能受 GitHub 限流，但 updater 下载与检查独立于本能力，保持可升级；不要为了回滚强制新版 App 重新直连 GitHub。

## Risks and Acceptance

- 网站成为公共版本历史入口，但已有下载与 updater 也使用同域名；本次不额外增加部署服务。站点故障不得影响本地剪贴板操作。
- CDN 缓存允许 5 分钟发布延迟；说明编辑/撤回允许最多 6 小时补偿同步延迟加缓存时间。
- 单个 releases JSON 会比当前每页 10 条响应大，但由 CDN 分发且当前数据规模小；如果体积变成真实问题，再新增 v2 分页。
- 验收必须覆盖同步失败保留旧数据、恶意 HTML、跨域正式 App 访问、官网 changelog/home 无 GitHub 客户端请求，且 header 不再请求 GitHub stars。只通过 mock 测试不能视为上线完成。
