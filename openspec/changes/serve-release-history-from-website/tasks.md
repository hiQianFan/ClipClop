## 1. 用户确认与契约准备

- [x] 1.1 获得用户对本提案的确认后才开始以下实施任务。
- [ ] 1.2 在 website 仓库创建对应 OpenSpec 变更并引用本提案；确认现有 production-release environment 允许计划任务使用已有最小权限凭据。
- [x] 1.3 固定 `releases.json` schema、路径、content hash、缓存与 CORS 契约；准备覆盖双语正文、空历史、客户端显示边界和恶意 HTML 的 fixture。

## 2. 主仓库集中同步

- [x] 2.1 实现认证、有界 GitHub 分页抓取，过滤草稿与预发布，稳定排序和去重；获取 Markdown 和渲染正文。
- [x] 2.2 校验 GitHub 渲染 HTML 的安全边界与 metadata；生成单个 `releases.json`。
- [ ] 2.3 实现 dry-run、内容未变化跳过、上传校验、最后替换正式文件；记录旧文件供回滚。
- [x] 2.4 建立共享生成脚本，接入发布成功后的显式同步、手动触发与六小时 schedule。
- [ ] 2.5 测试跨 GitHub 页完整抓取、过滤、首次回填、正文编辑、撤回、非法数据、超限、失败不替换文件与并发发布。

## 3. Website 静态分发

- [x] 3.1 在 worker.js / wrangler.toml 增加精确匹配的 `/releases.json` 路由，复用现有 R2 只读 binding。
- [x] 3.2 实现 GET/HEAD/OPTIONS、JSON MIME、ETag、CORS、404/405/503 和 no-store 错误策略。
- [ ] 3.3 配置并验证 `releases.json` 的五分钟缓存与实际边缘缓存行为；条件请求和 HEAD 不污染 GET 缓存。
- [x] 3.4 添加 Worker 路由、错误 CORS、缓存头与 R2 故障测试；通过 website 现有 test/build 检查。

## 4. 消费方迁移

- [x] 4.1 App 改用 website `releases.json`，验证 schema 与字段边界，保留会话缓存、请求去重、选择保持和过期响应隔离；本地按 10 条递增显示。
- [x] 4.2 App 配置精确 CSP 域名及通用站点错误提示；测试首次错误/骨架切换、追加错误保留、手动刷新与恶意响应。
- [x] 4.3 官网 changelog 切换相同 `releases.json`，保留当前语言提取与缓存降级，升级缓存 key。
- [x] 4.4 官网首页 stable version 文案切换 `releases.json`；header 移除动态 stars 请求，保留 GitHub 链接。
- [x] 4.5 验证 App 与官网版本历史、首页、header 不再直连 GitHub，不携带私密凭据，现有更新检查/下载/安装契约未变化。

## 5. 回填、发布与验收

- [x] 5.1 更新两仓库 distribution 文档，包含同步失败处理及 Token/凭据所有权。
- [ ] 5.2 完成一次真实历史 dry-run 和 R2 回填，检查 `releases.json` 可达；用户批准实施后按既有生产流程部署 website 路由。
- [ ] 5.3 公网验证 CORS、GET/HEAD/OPTIONS、ETag/CDN 和 `releases.json`，再切换官网与发布 App。
- [ ] 5.4 macOS 与 Windows 正式构建实测首次加载、键盘/滚动显示、断网/超时重试、手动刷新；验证 CSP 无阻断。
- [ ] 5.5 演练同步失败仍可访问旧 `releases.json` 和恢复旧文件；记录证据后完成 OpenSpec 验收。
