# ClipClop v1 实施计划

状态：待实施  
依据：`PRODUCT.md`、`DESIGN.md`、`docs/architecture.md`、PRD 文档集与 `outputs/prototype/clipclop-dark.html`

## 1. 规范优先级与冲突裁决

1. 产品范围以 `outputs/prds/prd-ClipClop-2026-07-10/prd.md` 及 `addendum.md` 中明确的 v1 要求为准。
2. 架构边界以 `docs/architecture.md` 为准：Tauri 2 + Rust 模块化单体 + SQLite + Svelte 5。
3. 布局、密度与视觉构成以已确认的 dark prototype 为主要参照。
4. Light/Dark 色值以根目录 `DESIGN.md` 为准。早期附录中的玻璃、暖色外壳、彩色类型图标及永久筛选控件均被后续灰阶设计取代。
5. 文档中的产品名统一为 `ClipClop`；历史产出文件保留原始决策记录，但正式文档不再使用 `Clip-Clop`。

## 2. 交付范围

### 2.1 Rust 应用后端

- 应用状态、稳定错误码和类型化 IPC DTO。
- SQLite migrations、clips/flavors/settings/ignored apps 数据表、FTS 搜索与分页。
- 文本、富文本、链接、颜色、图片和文件引用的数据模型与持久化。
- 剪贴板轮询监听、去重、concealed/transient 跳过、暂停记录与保留期清理。
- rich/plain 写回、copy-only 与 direct-paste 编排及权限失败回退。
- pin/unpin、单条删除、清空非 pinned 历史、设置与忽略来源应用。
- 全局快捷键、Quick Panel 窗口生命周期、系统托盘与启动时初始化。
- macOS/Windows 平台适配边界；当前平台实现并编译验证，另一平台保持条件编译可构建。

### 2.2 Svelte 前端

- 唯一 IPC client、DTO、events 生命周期与错误恢复。
- Quick Panel：搜索、10 条分页列表、选择、预览、固定底栏、操作菜单。
- 文本/代码/链接/颜色/图片/文件预览；不渲染不受信任 HTML，不联网抓取 URL 元数据。
- 键盘操作：上下选择、左右翻页、1–0 跳行、Enter 复用、纯文本复制、搜索、操作菜单与 Esc。
- loading、empty、search-empty、error、retry、disabled、success 状态。
- 设置页面：快捷键、开机启动、保留期、直接粘贴、暂停捕获、ignored apps。
- macOS 首次权限说明与 copy-only 回退；Windows 不展示无意义的权限门槛。
- 响应默认 720×540 与最小 640×480；Dark/Light 跟随系统并支持减少动态效果。

### 2.3 设计系统

- `tokens.css` 完整落地 `DESIGN.md` 的 Light/Dark token、字体、间距、圆角、焦点与 motion。
- UI 产品控件保持灰阶；只有用户内容可保留色彩。
- prototype 的 300px 左栏、右侧预览、48px 状态栏、44px 行高和内容槽结构。
- WCAG AA 对比度、清晰焦点、44×44 可操作目标和完整键盘可达性。

### 2.4 测试与发布

- Rust 单元测试：类型推断、规范化、去重、rich/plain、保留期与输入限制。
- SQLite 集成测试：migration、事务、分页、搜索、pin、删除和设置持久化。
- 前端单元测试：键盘映射、状态转换、IPC 错误处理与列表行为。
- 完整质量门：Svelte check、前端测试、构建、Rust fmt/clippy/test。
- 当前 macOS 生成可安装 `.dmg`/`.app`；Windows 配置与 CI 构建 `.msi`/NSIS。
- 关键运行路径进行真实 Tauri 冒烟验证。

### 2.5 开源级文档

- 完整 README：截图、能力、隐私、安装、开发、构建、测试、故障排查和路线图。
- 完整 PRD：目标用户、范围、用户旅程、FR/NFR、验收标准、风险与发布标准。
- 完整设计规范：token、组件、状态、响应式、无障碍和 prototype 对照。
- 完整架构：模块、数据模型、IPC、并发、平台差异、安全、测试与 ADR。
- `CONTRIBUTING.md`、`SECURITY.md`、`PRIVACY.md`、`LICENSE`、行为准则、变更日志与发布指南。
- 文档索引、测试指南、平台权限指南和维护者发布清单。

## 3. 实施顺序与相对工作量

1. **工程与提交基线（小）**：清理 ignore、统一命名、补测试框架和开发配置，创建初始提交。
2. **领域模型与 SQLite（中）**：migration、模型、存储、分页/搜索、设置；先测后接 UI。
3. **剪贴板与平台能力（中）**：捕获、过滤、去重、写回、全局快捷键和窗口生命周期。
4. **类型化 IPC（小）**：commands、events、错误契约与前端 client。
5. **Quick Panel UI（中）**：tokens、列表、预览、键盘、操作菜单与完整状态。
6. **Settings 与 onboarding（中）**：配置持久化、ignored apps、权限与回退。
7. **集成与质量修复（中）**：端到端路径、无障碍、性能、异常与真实运行验证。
8. **开源文档与发布（中）**：完善文档、CI、图标/元数据、安装包和发布检查。

每个步骤完成对应测试后形成独立本地提交，避免堆积大改动。

## 4. 验收标准

### 功能

- [ ] 支持的剪贴板变化会入库并在重启后保留。
- [ ] 支持文本、富文本、链接、颜色、图片和文件引用；未知格式失败不会停止监听。
- [ ] concealed/transient 项不落盘；应用不上传或记录剪贴板正文。
- [ ] 短时间重复内容按规范去重。
- [ ] 全局快捷键打开 Quick Panel，搜索可立即使用。
- [ ] 历史最新优先、pinned 置顶、10 条分页、搜索和类型查询正确。
- [ ] 选择项具有安全且类型匹配的预览与来源/类型/大小/时间元数据。
- [ ] 默认 rich 写回保留已有 flavor；plain 模式只写纯文本。
- [ ] copy-only 始终可用；direct paste 不可用时给出可恢复提示。
- [ ] pin、unpin、删除、清空非 pinned、暂停/恢复均持久化且刷新 UI。
- [ ] 快捷键、开机启动、保留期、直接粘贴和 ignored apps 设置重启后保持。
- [ ] ignored app 的后续复制不被捕获，移除后恢复。

### UI 与交互

- [ ] 默认 720×540、最小 640×480 时无溢出，布局与 prototype 一致。
- [ ] Light/Dark 使用 `DESIGN.md` 指定 token，信息层级一致且没有额外品牌色。
- [ ] 文本/代码无类型图标；颜色、图片、文件和本地 favicon 使用固定内容槽。
- [ ] hover、selected、focus、active、disabled、loading、error、empty、success 状态完整。
- [ ] 上下、左右、1–0、Enter、搜索、操作菜单和 Esc 均按规范工作。
- [ ] 全部交互可由键盘完成，焦点清晰，语义标签与对比度满足 WCAG AA。
- [ ] 不渲染 HTML、不主动请求复制 URL、不生成摘要或内容解释。

### 工程、发布与文档

- [ ] `pnpm check`、前端测试、`pnpm build` 全部通过。
- [ ] `cargo fmt --check`、`cargo clippy -D warnings`、`cargo test` 全部通过。
- [ ] 本机 Tauri 开发运行与生产安装包构建成功。
- [ ] macOS 安装包可启动；Windows CI 配置能生成对应安装包。
- [ ] Git 历史按工程基线、后端、平台、前端、设置、测试、文档、发布合理分批。
- [ ] README、PRD、设计、架构、贡献、安全、隐私、许可证、变更记录和发布文档完整且互相一致。

## 5. 已知约束

- Windows 安装包与系统剪贴板行为最终需要 Windows runner/实机证明；本机只能验证条件编译配置与 macOS 产物。
- macOS direct paste 依赖 Accessibility 权限，拒绝授权时必须退化为 copy-only。
- 文件不被复制进应用数据目录；文件移动后显示引用失效。
- v1 不包含云同步、账号、AI、插件、标签/文件夹或联网内容增强。

## 6. 实施前环境基线（2026-07-13）

- Node.js：项目指定的 `v24.16.0` 已通过本机 nvm 验证；非交互 shell 需要显式加载 `~/.nvm/nvm.sh`。
- pnpm：通过 Corepack 使用项目锁定的 `9.15.3`，`pnpm install --frozen-lockfile` 成功。
- Rust：stable `rustc/cargo 1.96.0`，目标 `aarch64-apple-darwin`。
- Apple 工具链：Xcode Command Line Tools 路径有效。
- 前端开发服务器：Vite 在 `127.0.0.1:1420` 正常启动，根页面可请求。
- 设计资料：`DESIGN.md`、design spec、dark prototype 与 mock data 均可访问；尚无生产 token/component 文件，属于计划内工作。
- `pnpm check`：通过，0 errors / 0 warnings。
- `pnpm build`：通过，静态产物写入 `build/`。
- `cargo fmt --check`：通过。
- `cargo clippy --all-targets -- -D warnings`：通过。
- `cargo test`：通过，但当前 Rust 与前端均没有业务测试；这是既有基线，不代表功能已实现。
- 当前页面与 Rust command 仍是 create-tauri-app 默认模板；这属于已知产品缺口而非回归。
