# Windows 非激活面板 dev 验证

状态：实验实现，尚无 Windows 实机通过记录，不是完整修复或发布版本。
macOS 的 WPS 搜索框和 Quick Look 回归已由用户实测通过。

## 启动

在 Windows PowerShell、项目根目录中：

```powershell
git pull --ff-only origin main
pnpm install --frozen-lockfile
$env:CLIPCLOP_WINDOWS_NOACTIVATE = "1"
pnpm tauri dev
```

完全退出旧 ClipClop 再启动，避免单实例逻辑把请求交给旧进程。
日志出现 `Windows nonactivating dev panel shown` 表示进入实验路径。
此开关仅在 debug 构建有效。没有改版本号、创建标签或运行发版工作流。

恢复现有 Windows 默认行为：停止 dev，执行以下命令后重新启动。

```powershell
Remove-Item Env:CLIPCLOP_WINDOWS_NOACTIVATE
pnpm tauri dev
```

## 实现与已知限制

- 主面板、快捷面板使用 `WS_EX_NOACTIVATE`、`MA_NOACTIVATE` 和非激活显示。
- 专用线程安装鼠标和键盘低级钩子。面板隐藏时不路由新按键；关闭前已拦截的按键继续吞掉重复及松开事件，避免落入原应用。
- 键盘通过队列交给 WebView2 `Input.dispatchKeyEvent`，使用现有网页控件和事件处理；不通过全局 `SendInput` 向面板输入。
- 外部点击、前台窗口变化、Win 或 Alt+Tab/Alt+Esc 切换结束会话；队列和 COM 回调校验会话，旧事件不能操作新面板。
- 实验路径粘贴要求原目标仍在前台；不主动把切走的目标拉回来。粘贴按键注入不经过面板键盘路由。
- **尚未实现 OS 中文 IME 组合输入、候选框与死键组合。** 当前验证范围是非激活交互及普通字符/导航输入。AltGr、多布局、Caps Lock、辅助技术和复杂菜单也不能宣称完整兼容。不能用拼音字母搜索代替中文输入法验收。
- WebView2 内部是否会在点击输入控件时重新激活窗口、CDP 输入是否完整处理编辑快捷键，都需要实机验证；若激活发生，本实现会结束会话，不能把这种情况算作成功。
- 钩子失败、队列拥塞或协议调用失败应关闭实验会话；只记录错误，不记录按键文本、剪贴板内容或目标文档标题。

ClipboardX 提供了非激活窗口与键盘钩子的参考思路；本项目使用 WebView2，输入桥接独立实现。参考：[ClipboardX](https://github.com/chaojimct/clipboardx)、[微软低级键盘钩子文档](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)。

## 实机验收

请用空白测试文档，记录 Windows、WebView2、Chrome、WPS 和输入法版本。

1. Chrome WPS 网页查找框聚焦，呼出 ClipClop，点击条目、空白区域、搜索框、滚动条，再 Esc；不点鼠标直接输入，必须仍进入查找框。
2. 同样操作后 Enter/双击粘贴，必须只进入查找框，不能改动单元格；主面板和快捷面板分别重复十次。
3. 检查方向键、Enter、Shift+Enter、Esc、Space 预览、Tab/Shift+Tab、搜索输入、选中替换、删除、Ctrl+A/C/V、右键菜单及面板互切。
4. 长按 Enter/Esc 直到窗口消失，确认后续重复键和松开事件没有落入原应用；关闭后新输入必须立即恢复正常。
5. 面板打开时点击外部、Alt+Tab、Win 切换应用；面板应隐藏，新应用输入正常，待处理粘贴不能拉回旧应用。
6. 中文 IME 单独记录为未实现，不能勾选通过；正常默认路径仍需维持原有中文输入能力。
7. 对照桌面 WPS、记事本、浏览器普通输入框；检查管理员权限差异、QuickLook、设置窗口及拖动。

只有中文输入和完整控件行为都得到解决并通过实机回归，才能将本路径提升为默认实现。交叉编译及 CI 通过只证明编译和单元测试，不证明窗口焦点行为。
