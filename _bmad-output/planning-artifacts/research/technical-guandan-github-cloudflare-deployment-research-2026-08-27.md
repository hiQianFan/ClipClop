---
stepsCompleted: [1, 2, 3, 4]
inputDocuments: []
workflowType: research
lastStep: 4
research_type: technical
research_topic: guandan GitHub 到 Cloudflare 的标准部署与统一域名出口
research_goals: 评估 Pages 与 Workers，形成可持续部署、预览、回滚及单一公开入口方案
user_name: qianfan
date: 2026-08-27
web_research_enabled: true
source_verification: true
---

# 技术调研：guandan GitHub → Cloudflare 部署

## Technical Research Scope Confirmation

评估架构选择、GitHub 持续部署、域名暴露、预览、回滚、权限及迁移成本；所有平台能力以 Cloudflare 当前官方文档和线上项目配置为准。

## Technology Stack Analysis

### 当前应用与构建栈

`hiQianFan/guandan` 是纯前端 Vue 3 + TypeScript + Vite 应用，`npm run build` 输出 `dist`。仓库未显示服务端、数据库或运行时绑定需求，因此部署目标本质上是静态资源与 SPA fallback，不需要为部署额外编写 Worker 业务代码。

### 当前 Cloudflare 栈

线上资源是 GitHub 集成的 Pages 项目 `guandan`：生产分支 `main`，构建命令 `npm run build`，输出目录 `dist`。公开域名包括 `guandan.mapin.net`、`guandan-bze.pages.dev`，且当前所有分支均启用公开 Preview Deployment；最近一次生产部署另有持久化 hash 地址。

Pages 的优势是 GitHub 集成、PR 预览与生产回滚均为平台原生能力；但 `pages.dev` 没有直接关闭开关。官方方案是将生产 `pages.dev` 通过 Bulk Redirect 转到自定义域名，并用 Access 保护预览地址。[Pages Git 集成](https://developers.cloudflare.com/pages/configuration/git-integration/) · [自定义域名与 pages.dev 控制](https://developers.cloudflare.com/pages/configuration/custom-domains/) · [预览部署](https://developers.cloudflare.com/pages/configuration/preview-deployments/)

### 候选目标栈

Workers Static Assets 可以直接托管现有 `dist`，SPA 使用 `assets.not_found_handling: "single-page-application"`，无需新增应用服务器或容器。它支持显式设置 `workers_dev: false` 和 `preview_urls: false`，再把 `guandan.mapin.net` 配为 Custom Domain，因此能满足“公网只保留主域名”的硬要求。[Static Assets](https://developers.cloudflare.com/workers/static-assets/) · [从 Pages 迁移](https://developers.cloudflare.com/workers/static-assets/migration-guides/migrate-from-pages/) · [Wrangler 配置](https://developers.cloudflare.com/workers/wrangler/configuration/)

### GitHub 部署工具

Workers 有两条标准路径：Cloudflare Workers Builds 原生连接 GitHub，或 GitHub Actions 调用 `cloudflare/wrangler-action@v3`。本项目部署流程简单，没有已知审批链或多环境编排需求，因此原生 Workers Builds 是组件最少的默认方案；仓库中的 Wrangler 配置应作为部署真源。若未来必须在部署前强制执行浏览器测试或人工审批，再切到 GitHub Actions，避免两套发布器并存。[Workers Builds](https://developers.cloudflare.com/workers/ci-cd/builds/) · [GitHub Actions](https://developers.cloudflare.com/workers/ci-cd/external-cicd/github-actions/)

### 初步技术判断

若“统一入口”只表示用户最终停留在主域名，保留 Pages 并加 Redirect + Access 最省迁移；若要求其他 Cloudflare 默认地址不能匿名访问，Workers Static Assets 是更符合约束的实现。当前应用与 Workers Static Assets 高度兼容，迁移不需要数据库、容器或服务端框架。

## Integration Patterns Analysis

### GitHub → Cloudflare

推荐只保留 Workers Builds 一个发布器。`main` push 依次执行依赖安装、`npm test`、`npm run build`、`npx wrangler deploy`；Worker 名称必须与仓库 Wrangler 配置中的 `name` 一致。关闭 non-production branch builds，PR 只由 GitHub CI 执行测试和构建，不上传公开预览版本。[Workers Builds](https://developers.cloudflare.com/workers/ci-cd/builds/) · [Build branches](https://developers.cloudflare.com/workers/ci-cd/builds/build-branches/)

### 仓库部署契约

仓库应提交最小 `wrangler.jsonc`，声明静态资源目录 `dist`、SPA fallback、`workers_dev: false`、`preview_urls: false`，以及 `guandan.mapin.net` Custom Domain。构建命令仍是现有 `npm run build`。Custom Domain 由 Cloudflare 创建受管 DNS 与证书，适合 Worker 本身作为源站的纯静态应用。[Custom Domains](https://developers.cloudflare.com/workers/configuration/routing/custom-domains/)

### 迁移切换

不能让 Pages 与 Worker 同时占用 `guandan.mapin.net`。标准顺序是：先以临时的受限测试 hostname 或本地 Wrangler 验证 Worker；记录现有 Pages 部署；从 Pages 移除 `guandan.mapin.net`；立即部署带同名 Custom Domain 的 Worker；核对 TLS、SPA 深链和静态资源；最后删除 Pages 项目。切换期间保留 Pages 默认地址只作为短暂回退，不把它作为正式入口。[Pages → Workers 迁移](https://developers.cloudflare.com/workers/static-assets/migration-guides/migrate-from-pages/)

### 发布与回滚

每个成功部署形成 Worker 版本；失败构建不改变生产流量。生产异常时，通过 Dashboard 或 `wrangler rollback <VERSION_ID>` 将旧版本恢复到全部 Domain/Route。Workers 保留最近 100 个可回滚版本；本项目没有 KV、D1 或 Durable Objects，当前不存在数据绑定阻碍代码回滚的问题。[Workers rollback](https://developers.cloudflare.com/workers/versions-and-deployments/rollbacks/)

### 权限与安全边界

Workers Builds 使用单一、最小权限构建 Token，限制到该账户及 `mapin.net` Zone；仓库不保存 Cloudflare Token。生产分支启用 GitHub branch protection，并要求测试和构建检查通过。由于应用纯静态，不引入 Secrets、KV、消息队列、API Gateway 或服务间通信。

## Architectural Patterns and Design

### 目标架构

```text
GitHub: hiQianFan/guandan (main)
          │ push
          ▼
Cloudflare Workers Builds
  npm test → npm run build → wrangler deploy
          │
          ▼
Workers Static Assets (guandan)
          │ Custom Domain only
          ▼
https://guandan.mapin.net
```

这是单仓库、单生产环境、单发布器、单公网 hostname 的静态边缘部署。Cloudflare 原生提供构建、资产分发、CDN、TLS、版本和回滚，不增加 Worker 脚本、源站服务器或对象存储。[Static Assets](https://developers.cloudflare.com/workers/static-assets/)

### 核心架构决策

1. **Workers Static Assets 替代 Pages**：满足默认平台域名不可公开的约束；Pages 只能重定向或通过 Access 保护 `pages.dev`。
2. **Custom Domain 替代 Zone Route**：Worker 就是源站，没有外部 origin；Cloudflare 官方也将 Custom Domain 推荐给这种场景。[Routes and domains](https://developers.cloudflare.com/workers/configuration/routing/)
3. **配置入库**：`workers_dev: false` 必须写进 Wrangler 配置，仅在 Dashboard 关闭会被后续部署重新启用；`preview_urls: false` 同样显式声明。[workers.dev](https://developers.cloudflare.com/workers/configuration/routing/workers-dev/)
4. **不写 Worker runtime**：`assets.directory` 与 SPA fallback 已覆盖 Vue 应用；匹配的静态资产直接由平台提供并自动 CDN 缓存。[SPA routing](https://developers.cloudflare.com/workers/static-assets/routing/single-page-application/)
5. **不建 staging**：当前无后端或数据迁移风险。PR 跑测试与构建，生产版本可即时回滚，足够覆盖风险。

### 最小仓库配置

```jsonc
{
  "$schema": "node_modules/wrangler/config-schema.json",
  "name": "guandan",
  "compatibility_date": "2026-08-27",
  "workers_dev": false,
  "preview_urls": false,
  "assets": {
    "directory": "./dist",
    "not_found_handling": "single-page-application"
  },
  "routes": [
    { "pattern": "guandan.mapin.net", "custom_domain": true }
  ]
}
```

Wrangler 应锁定为仓库 devDependency，避免构建环境隐式使用不同版本。除此之外无需新增运行时代码、Bindings 或部署脚本。

### 性能、安全与运维

静态资源由 Workers Static Assets 自动使用 Cloudflare 缓存和分层缓存；无需手写 Cache API。TLS 与 DNS 由 Custom Domain 管理。生产分支受保护，构建 Token 不进入仓库。每次部署形成不可变版本，异常时回滚到最近成功版本。[Workers rollback](https://developers.cloudflare.com/workers/versions-and-deployments/rollbacks/)

### 架构结论

推荐方案是 **Workers Static Assets + Workers Builds + Custom Domain only**。它比当前 Pages 多一个很小的 Wrangler 配置，却原生满足单一公开出口；没有必要通过 Redirect、Access 或额外 Worker 去掩盖 Pages 默认域名。
