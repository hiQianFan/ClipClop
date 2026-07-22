<div align="center">
  <img src="src-tauri/icons/icon.png" width="132" alt="ClipClop 应用图标">

  # ClipClop

  **马不停贴。**

  剪贴历史，一键即达。

  [![构建状态](https://github.com/hiQianFan/ClipClop/actions/workflows/quality.yml/badge.svg)](https://github.com/hiQianFan/ClipClop/actions/workflows/quality.yml)
  ![支持 macOS](https://img.shields.io/badge/macOS-支持-000000?logo=apple&logoColor=white)
  ![支持 Windows](https://img.shields.io/badge/Windows-支持-0078D4?logo=windows11&logoColor=white)
  [![Stars](https://img.shields.io/github/stars/hiQianFan/ClipClop?style=flat)](https://github.com/hiQianFan/ClipClop/stargazers)
  [![开源许可](https://img.shields.io/github/license/hiQianFan/ClipClop)](LICENSE)

  [下载](https://github.com/hiQianFan/ClipClop/releases) · [产品亮点](#快不必以隐私为代价) · [隐私](#隐私与安全公开可验证) · [English](README.md)
</div>

## 不再弄丢复制过的内容

ClipClop 记住你复制过的文本、链接、颜色、图片和文件。按下快捷键，就能随时呼出、快速找到并粘贴回去，不打断手上的工作。

剪贴核心功能完全离线，内容保存在本机。无需账号，没有遥测、广告、云同步或复杂工作区——ClipClop 只补上电脑本该拥有的剪贴历史。

## 快，不必以隐私为代价

| | |
| --- | --- |
| ⚡ **快而不扰** | 覆盖在当前应用上方，需要时立即出现，完成后自然退场。 |
| ⌨️ **全程键盘操作** | 从搜索、选择、预览到粘贴和设置，核心流程无需鼠标。 |
| ♿ **无障碍优先** | 提供语义化控件、清晰焦点、状态播报和减少动态效果支持。 |
| 📴 **核心功能离线运行** | 捕获、保存、搜索、预览和粘贴都在本机完成。 |
| 🔒 **内容私密可控** | 没有账号、遥测、广告或云剪贴板，复制网址也不会触发后台访问。 |
| 🎨 **不只有纯文本** | 保存文本及其可用格式，也支持链接、颜色、图片和文件。 |

## 三步找回剪贴内容

1. **呼出**——macOS 按 `⌃⌘C`，Windows 按 `Ctrl+Alt+C`。
2. **查找**——直接输入搜索，或浏览最近复制的内容。
3. **粘贴**——按 `Enter` 保留可用格式，按 `Shift+Enter` 粘贴纯文本。

全局快捷键可以自定义。如果系统不允许直接粘贴，ClipClop 会把选中内容留在系统剪贴板，你仍可正常手动粘贴。

## 下载

前往 [GitHub Releases](https://github.com/hiQianFan/ClipClop/releases) 下载最新预览版：

- **macOS：**同时支持 Apple Silicon 与 Intel 的 Universal DMG
- **Windows：**x64 安装程序

> [!NOTE]
> 当前预览安装包尚未使用 Apple Developer ID 或 Windows Authenticode 签名，操作系统可能显示安全确认。应用更新文件另有完整性签名。

## 隐私与安全，公开可验证

ClipClop 没有账号、遥测、广告、云剪贴板或联网内容增强。剪贴内容的捕获、保存、搜索、预览和粘贴均在本机完成；除可关闭的更新检查外，ClipClop 不需要网络连接。即使复制的是网址，也不会在后台访问它。

ClipClop 完全开源，任何人都可以审查剪贴内容如何处理、数据保存在哪里，以及应用何时访问网络。开源本身不等于绝对安全，但它让隐私边界和安全承诺可以被验证，而不只是被相信。

你可以删除单条记录、清空历史、设置保留期限，或退出 ClipClop 来停止捕获。准确的数据处理方式见[隐私说明](PRIVACY.zh-CN.md)。

## 项目状态

ClipClop 当前为 `0.1.1` 预览版。macOS 与 Windows 构建均接受持续检查，更广泛的实机测试仍在进行中。欢迎通过 [GitHub Issues](https://github.com/hiQianFan/ClipClop/issues) 提交反馈与问题。

<details>
<summary><strong>参与开发</strong></summary>

开发环境、项目约定和安全问题私密报告入口统一放在[贡献指南](CONTRIBUTING.zh-CN.md)中。

</details>

## 开源许可

ClipClop 采用 [MIT License](LICENSE) 开源。如果它对你有帮助，欢迎点亮 ⭐，让更多人发现它。
