<div align="center">
  <img src="src-tauri/icons/icon.png" width="132" alt="ClipClop 应用图标">

  # ClipClop

  **剪贴历史，一键即达。**

  为 macOS 与 Windows 打造的快速、私密剪贴板历史工具。

  [![构建状态](https://github.com/hiQianFan/ClipClop/actions/workflows/quality.yml/badge.svg)](https://github.com/hiQianFan/ClipClop/actions/workflows/quality.yml)
  ![支持平台](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-555)
  [![Stars](https://img.shields.io/github/stars/hiQianFan/ClipClop?style=flat)](https://github.com/hiQianFan/ClipClop/stargazers)
  [![开源许可](https://img.shields.io/github/license/hiQianFan/ClipClop)](LICENSE)

  [下载](https://github.com/hiQianFan/ClipClop/releases) · [产品亮点](#为什么选择-clipclop) · [隐私](#隐私不是附加功能) · [English](README.md)
</div>

## 不再弄丢复制过的内容

ClipClop 安静地记住你复制过的文本、链接、颜色、图片和文件。无论正在使用什么应用，都能随时呼出、快速找到，并粘贴回去，不打断手上的工作。

无需账号，没有云同步，也不把剪贴板包装成信息流、工作区或 AI 工具。它只是补上电脑本该拥有的剪贴历史。

## 为什么选择 ClipClop

| | |
| --- | --- |
| ⚡ **一个快捷键就到** | 覆盖在当前应用上方，立即搜索最近复制过的内容。 |
| ⌨️ **为键盘操作而生** | 查找、预览、复制、粘贴，全程不必离开键盘。 |
| 🎨 **不只有纯文本** | 保存文本及其可用格式，也支持链接、颜色、图片与文件引用。 |
| 🖥️ **真正面向双平台** | 为 macOS 与 Windows 提供专注、自然的桌面体验，并支持明暗主题。 |
| 🔒 **默认留在本地** | 剪贴历史保存在当前设备，复制链接也不会触发后台联网请求。 |
| 🪶 **安静而轻量** | 常驻托盘、不打扰，需要时才出现。 |

## 三步找回剪贴内容

1. **呼出**——macOS 按 `⌃⌘C`，Windows 按 `Ctrl+Alt+C`。
2. **查找**——直接输入搜索，或浏览最近复制的内容。
3. **粘贴**——按 `Enter` 保留可用格式，按 `Shift+Enter` 粘贴纯文本。

全局快捷键可以自定义。如果系统不允许直接粘贴，ClipClop 会把选中内容留在系统剪贴板，你仍可正常手动粘贴。

## 下载

首次版本发布后，可前往 [GitHub Releases](https://github.com/hiQianFan/ClipClop/releases) 获取预览版：

- **macOS：**同时支持 Apple Silicon 与 Intel 的 Universal DMG
- **Windows：**x64 安装程序

> [!NOTE]
> 当前预览安装包尚未使用 Apple Developer ID 或 Windows Authenticode 签名，操作系统可能显示安全确认。应用更新文件另有完整性签名。

## 隐私不是附加功能

ClipClop 没有账号、遥测、广告、云剪贴板或联网内容增强。剪贴板内容与设置留在当前设备；只有可关闭的更新检查会访问 GitHub Releases。

你可以删除单条记录、清空历史、设置保留期限，或退出 ClipClop 来停止捕获。准确的数据处理方式见[隐私说明](PRIVACY.zh-CN.md)。

## 项目状态

ClipClop 当前为 `0.1.0` 预览版。macOS 与 Windows 构建均接受持续检查，更广泛的实机测试仍在进行中。欢迎通过 [GitHub Issues](https://github.com/hiQianFan/ClipClop/issues) 提交反馈与问题。

<details>
<summary><strong>参与开发</strong></summary>

开发环境、项目约定和安全问题私密报告入口统一放在[贡献指南](CONTRIBUTING.zh-CN.md)中。

</details>

## 开源许可

ClipClop 采用 [MIT License](LICENSE) 开源。如果它对你有帮助，欢迎点亮 ⭐，让更多人发现它。
