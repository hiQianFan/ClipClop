---
title: '增加可重置的完整演示模式'
type: 'feature'
created: '2026-09-08'
status: 'done'
route: 'dispatch'
review_loop_iteration: 0
baseline_commit: 'ccc7de3188fd4dba2db9b24846cb440ac1951263'
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/docs/architecture.zh-CN.md'
  - '{project-root}/openspec/changes/add-resettable-demo-mode/'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 宣传录屏、现场体验和服务展示需要在真实 ClipClop 中完整操作，但现有快速入门只模拟部分行为，真实历史又可能为空或包含私人数据。

**Approach:** 增加一个进程内演示会话，在后端统一切换完整 History 数据源。演示复用所有既有 item API 和工作流，退出时销毁内存数据库与临时资源，下次进入从官网固定内容重新开始。

## Boundaries & Constraints

**Always:** 现有历史 IPC 不增加 mode 参数；History、Asset、External Preview 和 capture 原子解析同一环境；演示不读写真实历史；官网中英文 `demo.clips` 的 10 项是内容事实源；入口为“设置 → 通用 → 演示模式 → 进入…”；进入说明外部副作用；标题栏显示中性“演示模式”；仅演示期间应用菜单提供退出；隐藏面板不退出。

**Never:** 不借真实库模拟隔离；不复制 UI、搜索或工作流；不联网或运行时读取网站仓库；不增加 trait/factory、新依赖、Switch、启动参数、托盘入口或快捷键；不撤销外部应用副作用；不沙盒化 Settings、更新、开机启动或系统权限。

## I/O & Edge-Case Matrix

| 场景 | 输入 / 状态 | 预期行为 | 错误处理 |
|---|---|---|---|
| 进入 | 真实历史任意；确认“进入演示” | 创建内存 SQLite，按有效语言播种官网 10 项，清前端会话并选中第一项 | 创建/播种/切换失败保持真实模式，可重试 |
| 完整操作 | 搜索、筛选、预览、复制、粘贴、外链、删除或清空 | 使用原 API/outcome；修改仅在演示环境 | 原错误语义不变 |
| 新捕获 | 演示期间 watcher 观察到 clipboard snapshot | 写入演示库并发出既有 `history_changed`；不写真实库、不退出后回放 | 沿用 capture 错误处理 |
| 隐藏/呼回 | Escape、Command/Ctrl+W、失焦或全局快捷键 | 同一演示会话继续存在，状态标签仍显示 | 沿用窗口生命周期 |
| 退出 | 应用菜单选择“退出演示模式” | 等待在途操作，关闭预览，清临时资源，切回真实库，清前端状态并聚焦列表 | 清理失败保留演示环境与重试入口 |
| 重进/重启 | 修改后重进或结束进程 | 重进恢复 10 项；重启为真实模式 | 清理遗留 demo 临时目录 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/{state.rs,demo.rs}` -- 组装永久/演示 History 环境与生命周期锁。
- `src-tauri/src/commands/{history,preview,demo}.rs`, `workflows/capture.rs`, `lib.rs` -- 所有 item 路径、watcher 与生命周期 command 使用当前环境。
- `src-tauri/src/{history,storage,assets,preview}` -- 复用内存库、FTS、去重、资源与系统预览；真实 schema 不变。
- `/Users/qianfan/Desktop/Code/clipclop.io/src/features/home/i18n/{zh,en}.ts` -- fixture 事实源，只在实施时同步。
- `src/lib/history/{api.ts,HistoryWorkspace.svelte,AppTitleBar.svelte}` -- 模式 API、Session 清理、状态与退出。
- `src/lib/settings/{GeneralSettings.svelte,SettingsView.svelte}`, `src/lib/i18n/catalogs.ts` -- 设置入口、AlertDialog 与双语文案。

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/demo.rs`, `state.rs` -- 建立环境、官网 fixture、生命周期锁与进入/退出/清理。
- [x] `src-tauri/src/commands/{history,preview,demo}.rs`, `workflows/capture.rs`, `lib.rs` -- 将全部 item 路径和 watcher 接到当前环境，注册三个模式 command。
- [x] `src/lib/history/{api.ts,HistoryWorkspace.svelte,AppTitleBar.svelte}`, `src/lib/settings/{GeneralSettings.svelte,SettingsView.svelte}`, `src/lib/i18n/catalogs.ts` -- 实现设置入口、确认、状态、退出与完整会话清理。
- [x] 相关测试 -- 覆盖隔离、并发、重置、失败、焦点和窗口生命周期；回填 OpenSpec tasks。

**Acceptance Criteria:**
- Given 真实库含记录，when 进入演示并操作，then UI 只展示演示内容，真实内容、排序及最近使用时间不变。
- Given 演示被修改，when 退出再进入，then 精确恢复对应语言的官网 10 项且搜索、筛选、页码、选择、菜单、确认和资源缓存均为初始状态。
- Given 任一历史操作与退出竞争，when 两者完成，then 单次操作只访问一个环境，退出成功后旧演示环境不再产生写入。
- Given 演示模式隐藏或失焦，when 再次呼出，then 会话保持；只有显式退出或进程结束才丢弃。

## Implementation Notes

- 自动验证全部通过；macOS 与 Windows 的真实系统预览、自动粘贴和外部应用联动仍需发布前实机走查。

## Spec Change Log

## Review Triage Log

| 来源 | 结论 | 证据与处理 |
|---|---|---|
| blind-1 重进构建失败破坏旧会话 | medium | 已修：先完成独立目录中的 replacement，再持锁清理和交换；构建失败保留旧环境。 |
| blind-2 关闭预览与切换间存在竞态 | medium | 已修：`before_swap` 在同一 runtime 锁内关闭当前环境的原生预览。 |
| blind-3 未监听 `runtime_mode_changed` | false | 主窗口是唯一 `HistoryWorkspace`，进入/退出调用直接采用 command 返回值；隐藏呼回另以 `get_runtime_mode` 校准。 |
| blind-4 呼回校准未完全清状态 | medium | 已修：模式变化时补清展开态并恢复 browse；随后既有呼回流程负责页码、选择和聚焦。 |
| blind-5 设置保存与语言播种竞态 | medium | 已修：确认进入前等待 `save()` 成功。 |
| blind-6 后端切换后刷新失败被当作进入失败 | false | `syncFacets` 自行吞掉错误，`refreshAndFocus` 将读取错误写入界面且不抛出，因此不会进入此错误分支。 |
| blind-7 AlertDialog Action 提前关闭 | medium | 已修：异步主操作改为普通按钮，只在成功后关闭 dialog。 |
| blind-8 演示 source ID 无法解析图标 | false | Source icon API 已定义无图标 fallback；演示 source ID 用于稳定筛选，不承诺伪造品牌图标。 |
| blind-9 占位 `.dmg` 不是真实磁盘镜像 | low | 已接受：fixture 用于文件复制、派生和系统预览降级；打包大型安装镜像超出内置演示数据的必要范围。 |
| blind-10 测试未覆盖 reset/failure | medium | 已修：新增构建失败保留与清理失败重试测试。 |
| blind-11 无切换并发测试 | medium | 已修：新增在途 operation 阻塞 exit cleanup 的同步测试。 |
| edge-1 重进 build 失败 | medium | 与 blind-1 同一缺陷，已修并有失败路径测试。 |
| edge-2 多步清理可部分完成 | low | 接受：清理操作幂等，失败时 demo 环境和退出入口保留，重试会继续清理。 |
| edge-3 `panel_shown` 模式查询失败阻断路由 | medium | 已修：模式校准失败不阻断原 panel request。 |
| edge-4 验收关键路径覆盖不足 | medium | 已增加并发、失败重试、重建保留、确认焦点和演示菜单测试；平台集成项保留实机检查。 |
| gap-1 watcher capture 隔离无工作流测试 | low | runtime 解析与完整闭包由隔离/并发测试覆盖；watcher 需要平台 AppHandle，保留在实机演示检查中。 |
| gap-2 原子切换仅顺序测试 | medium | 已修：新增 channel 同步的并发退出测试。 |
| gap-3 清理失败保留与重试无测试 | medium | 已修：新增失败后 `is_demo` 保持 true 并成功重试退出的测试。 |
| gap-4 UI 完整 session reset 无交互测试 | medium | 已补入口确认/取消焦点、状态标签与退出动作；workspace reset 复用既有 session API，并由现有关闭/呼回测试覆盖。 |

## Verification

**Commands:**
- `cargo fmt --check --manifest-path src-tauri/Cargo.toml` -- Rust 格式正确。
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings` -- 跨 cfg 无警告。
- `cargo test --manifest-path src-tauri/Cargo.toml` -- 后端隔离与工作流测试通过。
- `pnpm check && pnpm test && pnpm build` -- Svelte 类型、组件测试与生产构建通过。
- `openspec validate add-resettable-demo-mode --strict` -- 提案严格校验通过。

**Manual checks (if no CLI):**
- macOS/Windows 验证搜索、预览、复制、自动粘贴/降级、外链、删除、隐藏/呼回、退出还原与重进。
