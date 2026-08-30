---
id: SPEC-windows-powertoys-peek-integration
companions:
  - implementation-details.md
sources:
  - ../../planning-artifacts/research/technical-windows-space-file-preview-experience-research-2026-08-30.md
---

> **规范契约。** 本 SPEC 与 `companions:` 中的文件共同定义待实现、测试和验收的完整范围。

# Windows PowerToys Peek 条件预览集成

## Why

Windows 当前把 Space 的“预览”回退为用默认应用打开文件，造成抢焦点和工作流中断。ClipClop 不承担自建或捆绑 Windows 文件渲染器的长期成本；Microsoft PowerToys Peek 是 Windows Space 文件预览的明确前置条件，未安装用户不具备该能力。

## Capabilities

- id: CAP-1
  intent: Windows 用户安装 PowerToys Peek 后，可以在 ClipClop 中按 Space 预览当前文件。
  success: 对存在且可访问的文件记录按 Space 会打开 Peek，且不会启动该文件的默认关联应用；主面板与 Quick 面板行为一致。

- id: CAP-2
  intent: 未安装或无法使用 Peek 的 Windows 用户不会被 Space 打断。
  success: Peek 不可用时，主面板与 Quick 面板均不提供 Space 预览入口，按 Space 不调用任何外部应用；后端直接调用预览命令也返回不可预览。

- id: CAP-3
  intent: Windows 用户可以了解 Peek 集成状态并进入可信安装入口。
  success: Windows 设置页显示已就绪、未安装或权限不可用状态；未安装状态只链接 Microsoft 官方 PowerToys 安装说明，安装后重新进入设置或面板即可被识别，无需重启 ClipClop。

- id: CAP-4
  intent: macOS Quick Look 行为不受 Windows 集成影响。
  success: macOS 的 Space 打开、再次 Space/Escape 关闭、临时预览文件和 `PreviewState` 生命周期保持现状，既有相关测试继续通过。

## Constraints

- Windows Space 不得回退到 `open_clip_file()`、`open_clip()` 或默认关联应用。
- PowerToys Peek 必须是可选外部依赖；ClipClop 不捆绑、不静默安装、不下载其二进制文件。
- ClipClop 不复制 PowerToys 安装产物，不从其源码剥离或维护 Peek 分支，也不提供替代 provider；未安装时直接缺省该能力。
- 第一版只向 Peek 传递历史记录中已经存在的真实文件路径；不得为文本、链接、颜色或剪贴板图片生成临时文件。
- 检测和启动必须在 Rust 后端完成；主面板、Quick 面板与设置页读取同一能力结果，前端不得自行猜测安装路径。
- 进程必须使用参数化启动而非 shell 命令拼接；路径不得进入前端、遥测或用户可见日志。
- 提升权限运行时不得启动 Peek，因为该模式可能导致 Peek 的 WebView2 预览失效。
- 不新增数据库字段或持久化偏好；能力由当前机器状态实时派生。

## Non-goals

- 不内置 PDF、Office、媒体或通用文件渲染器。
- 不宿主 Windows `IPreviewHandler`，不引入 WinUI 子应用。
- 不支持 Peek CLI 模式下的左右文件导航、批量选择或窗口控制。
- 不自动修改 PowerToys 设置或快捷键。
- 不支持非官方安装目录的全盘搜索或用户自定义可执行文件路径。
- 不把 QL-Win QuickLook 纳入本次实现。
- 不把默认应用、Web 内置查看器或其他第三方工具作为 Peek 缺失时的兜底方案。

## Success signal

同一 Windows 构建在未安装 Peek 时按 Space 完全无副作用；安装并启用普通权限的 PowerToys 后，无需重启 ClipClop 即可从主面板和 Quick 面板用 Space 打开当前真实文件的 Peek 预览。任何检测或启动失败都不会打开默认应用，macOS Quick Look 保持原样。
