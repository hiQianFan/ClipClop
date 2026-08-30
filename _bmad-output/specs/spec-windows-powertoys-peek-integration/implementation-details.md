# 实施细节

## 1. 后端能力模型

在现有 `src-tauri/src/preview/platform.rs` 增加平台能力与启动逻辑，不新增 Preview 子模块。向前端暴露只含非敏感状态的 DTO：

```text
provider: macos_quicklook | powertoys_peek | unavailable
reason: null | not_installed | elevated
```

在 `commands/preview.rs` 增加 `get_preview_capability` 命令，并在 `lib.rs` 注册。每次查询实时检测，不缓存；两次 `is_file()` 的成本可忽略，且用户安装 PowerToys 后无需重启。

平台结果：

| 平台/状态 | provider | reason |
|---|---|---|
| macOS | `macos_quicklook` | `null` |
| Windows 普通权限且找到 Peek | `powertoys_peek` | `null` |
| Windows 未找到 Peek | `unavailable` | `not_installed` |
| Windows 提升权限且找到 Peek | `unavailable` | `elevated` |
| 其他平台 | `unavailable` | `not_installed` |

## 2. Windows 检测规则

仅按顺序检查 Microsoft 文档给出的默认位置：

1. `%LOCALAPPDATA%\PowerToys\WinUI3Apps\PowerToys.Peek.UI.exe`
2. `%ProgramFiles%\PowerToys\WinUI3Apps\PowerToys.Peek.UI.exe`

环境变量缺失时跳过对应候选；候选必须通过 `Path::is_file()`。不使用 `PATH`、注册表、递归搜索或固定 `C:`。把“候选路径生成与第一个有效文件选择”写成接收环境值和文件判断的纯函数，单测覆盖用户级优先、机器级回退、变量缺失和文件不存在。

提升权限检测使用现有 `windows-sys`，只补充所需的 `Win32_Security` feature，通过当前进程 token 的 elevation 信息判断。句柄必须在所有分支关闭；检测 API 失败时按不可用处理并记录不含文件路径的 warning。

## 3. 预览工作流

调整 `workflows/preview_clip.rs`，删除“原生预览不可用后调用默认应用”的 fallback 分支：

```text
macOS -> 现有 Quick Look toggle -> NativeOpened / NativeClosed
Windows + Peek 可用 + File -> 启动 Peek -> NativeOpened
其他情况 -> NotPreviewable
```

Windows 路径从 `HistoryService` 读取当前 `id/index`，要求内容类型为 `File`、索引有效、规范化路径存在且为文件。调用 `std::process::Command::new(peek_exe).arg(file_path).spawn()`，不经过 shell，不等待 Peek 退出。启动成功仅表示进程已创建；失败转换为 `AppError::Platform`，并记录 provider 与 OS error，不记录剪贴板文件路径。

Windows 不设置 macOS 专用 `PreviewState`：Peek 获得焦点后由自身处理 Escape/Space 关闭。`FallbackOpened` 从 Rust 枚举、TypeScript union 和 API 测试中移除，防止未来再次把“预览”解释为“打开”。明确打开链接等动作继续使用各自的显式命令，不受影响。

## 4. 主面板

`HistoryWorkspace.svelte` 初始化和每次 `main_panel_shown` 时刷新 capability，并派生：

```text
canPreviewSelected = macos_quicklook
  OR (powertoys_peek AND selected content_type == file)
```

- 只有 `canPreviewSelected` 时，浏览模式的 Space 才 `preventDefault()` 并调用 `viewSelectedClip()`；否则不处理该键。
- 操作菜单只在 `canPreviewSelected` 时显示预览项。
- macOS 文案保持“预览”；Windows 文案明确为“使用 PowerToys Peek 预览”。
- `NotPreviewable` 只恢复浏览焦点，不触发其他动作。

## 5. Quick 面板

`QuickPanel.svelte` 与主面板使用同一个 `getPreviewCapability()` API。`routeQuickKey` 增加 `canPreview` 输入；只有当前项为文件且 capability 为 `powertoys_peek`（或 macOS 原有能力）时，Space 才返回 preview action。

Quick 打开/刷新时重新查询 capability。能力查询失败按 unavailable 处理，不显示阻断错误；真正启动 Peek 失败才显示现有 inline error。不要为 Quick 单独增加安装提示。

## 6. 设置入口

在 `SettingsView.svelte` 的 Windows General 页增加一行原生按钮/静态状态，不引入新组件：

| 状态 | 说明 | 动作 |
|---|---|---|
| ready | 已检测到 PowerToys Peek，可在文件记录上按 Space 预览 | 无按钮或“了解 Peek” |
| not_installed | 安装 PowerToys Peek 后可启用 Space 文件预览 | “了解并安装” |
| elevated | ClipClop 正以管理员权限运行，Peek 集成不可用 | 无安装按钮 |

“了解并安装”使用现有 opener 打开 Microsoft 官方安装页：`https://learn.microsoft.com/windows/powertoys/install`。设置页每次 mount 调用 capability；从浏览器返回后窗口重新获得焦点时最多再检测一次，或在用户重新进入 General 页时检测。无需保存开关。

中英文 i18n 同步增加状态、菜单和错误文案；Windows 快捷键列表仅在 Peek ready 时显示 Space 预览，macOS 始终保留。

## 7. 安全与故障语义

- 所有入口以 Rust 后端校验为准，前端隐藏入口不是安全边界。
- 只执行两个检测到的固定文件名，不接受前端传入 executable path。
- 只传入数据库记录解析出的当前文件路径，不接受前端任意 path。
- Peek 缺失、索引无效、文件已删除、权限提升或能力变化：返回 `NotPreviewable`；进程创建失败：返回平台错误。
- 任何失败均不得调用默认应用。
- 日志允许记录 `provider=powertoys_peek`、状态和 OS error code；不得记录文件路径或剪贴板内容。

## 8. 测试与验证

### 自动化

- Rust：候选路径选择四种边界；非文件内容和缺失文件返回 `NotPreviewable`；fallback 不可达；wire values 稳定；权限检测句柄生命周期由 Windows 编译检查覆盖。
- TypeScript：`routeQuickKey` 在 capability false 时忽略 Space、true 时预览；主面板菜单按平台/内容类型显示；capability API DTO 映射。
- 现有 macOS preview、onboarding、Quick 键盘和 API 测试保持通过。
- 执行 `cargo test`、`pnpm test`、`pnpm check`，并至少进行一次 Windows target 编译检查。

### Windows 真机矩阵

1. 未安装 PowerToys：两面板 Space 无副作用，菜单无预览项，设置显示安装入口。
2. 用户级安装：主面板与 Quick 对单文件成功打开 Peek。
3. 机器级安装：同上，并验证检测回退。
4. 多文件记录：按当前索引预览，不提供 Peek 内左右导航承诺。
5. 文本、链接、颜色、剪贴板图片：Space 不启动 Peek。
6. 文件删除或移动：不启动默认应用，界面保持可操作。
7. Peek executable 存在但启动失败：显示错误且不 fallback。
8. 管理员权限启动 ClipClop：设置显示不可用，Space 不启动 Peek。
9. 安装 Peek 后不重启 ClipClop：重新进入设置或呼出面板后变为 ready。

## 9. 提交边界

建议一个功能提交完成闭环，避免先暴露无后端保障的前端入口：

1. `feat(windows): integrate optional PowerToys Peek previews`

若代码审查要求拆分，只允许先提交后端能力与 fallback 移除，再提交前端入口；两个提交在发版前必须同时存在。

## 10. 第三方分发边界

只允许调用用户机器上由 PowerToys 正常安装程序部署的 Peek。不得把 `PowerToys.Peek.UI.exe` 或其 DLL、WinUI 3 runtime、资源与第三方组件复制进 ClipClop 安装包；不得 fork、改名或自行签发 Peek。

虽然 PowerToys 主体采用 MIT License，但 Peek 不是单文件程序，依赖 `Peek.Common.dll`、`Peek.FilePreviewer.dll`、`PowerToys.Peek.UI.dll`、WinUI 3/WebView2 runtime、资源和第三方 NOTICE。自行分发会把构建、许可证清单、签名、安全更新与兼容责任转移给 ClipClop，因此明确排除。

设置中的安装入口只打开 Microsoft 官方说明页，由用户自行决定和完成安装。未安装时不提示下载独立二进制，不提供自动安装按钮，不尝试其他预览程序。

## 11. 同类产品引导模式

### Files

[Files 的第三方集成文档](https://files.community/docs/features/integrations) 将 PowerToys Peek 定义为便利性集成：用户已经安装 PowerToys 时无需额外配置，选择文件后直接按 Space；文档链接到 Microsoft Peek 说明，并明确第三方集成不由 Files 背书或负责。它没有把 Peek 复制进安装包，也没有在 Peek 缺失时切换到默认应用。

可复用原则：**检测到即零配置工作，未检测到则能力不存在；说明放在集成入口，而非首次启动流程。**

### Zotero QuickLook

[ZoteroQuickLook](https://github.com/mronkko/ZoteroQuickLook) 把 Windows Quick Look 工具列为明确前置条件，要求用户先安装能够接收文件路径参数的预览程序。该项目也暴露了反面经验：允许任意自定义 command 会引入路径配置、不同工具兼容和长期维护问题，而且该插件目前已停止活跃维护。

可复用原则：说明外部依赖，但不开放自定义 executable path，不承诺兼容多个 provider。

### ClipClop 引导决策

- 不弹首次启动 onboarding：预览不是 ClipClop 核心使用前置条件，不能打断所有 Windows 用户。
- 未安装时不吞掉 Space 后显示重复 toast：用户按 Space 可能有其他预期；能力不存在就不接管按键。
- 主面板和 Quick 的操作菜单不显示不可执行的预览项，避免制造“功能坏了”的印象。
- Windows General 设置页保留一条可发现的“PowerToys Peek 文件预览”集成状态；未安装时提供 Microsoft 官方说明链接。
- 安装后无需在 ClipClop 再启用开关，下一次能力刷新直接工作；这与 Files 的零配置模式一致。
- 帮助文案明确“由 Microsoft PowerToys 提供，需单独安装”，避免用户认为 Peek 随 ClipClop 分发或由 ClipClop 维护。
