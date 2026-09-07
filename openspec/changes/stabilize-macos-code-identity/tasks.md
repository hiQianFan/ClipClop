## 1. 创建并保管发布身份

- [x] 1.1 在受控 Mac 上创建名为 `ClipClop Release`、用途为 Code Signing 的长期自签名证书，导出加密 PKCS#12，并用 `security find-identity -v -p codesigning` 验证身份可用
- [ ] 1.2 按 `ClipClop/Credentials/{macOS,Updater}` 结构保存加密的 PKCS#12 与现有 updater 密钥到 iCloud Drive，并另存至少一份不同位置的加密离线备份；完成一次恢复验证并记录证书 SHA-256 指纹
- [x] 1.3 将 updater 密码条目命名为 `io.clipclop.desktop.updater`，确认密码不在 iCloud 文件目录中
- [x] 1.4 创建证书时新增密码条目 `io.clipclop.desktop.release-signing`，确认密码不在 iCloud 文件目录中
- [x] 1.5 在 GitHub `production-release` Environment 配置 PKCS#12 Base64、密码与证书指纹 Secrets，并验证普通 quality job 无法访问它们

## 2. 接入 macOS 发布流水线

- [x] 2.1 将应用 Bundle ID 及仓库内关联引用从 `com.clipclop.desktop` 统一为 `io.clipclop.desktop`，通过全仓搜索和构建产物 `Info.plist` 验证旧标识不再用于运行时身份
- [x] 2.2 在 macOS release job 中创建临时 Keychain、导入固定证书并授权 `codesign`，通过不输出敏感值的身份检查验证导入成功
- [x] 2.3 配置 Tauri universal macOS 构建使用固定身份，并用临时证书演练验证最终 App 的签名 Identifier 为 `io.clipclop.desktop` 且 `Signature` 不是 `adhoc`
- [x] 2.4 在 job 结束时无条件删除临时 Keychain，并通过 workflow review 确认 P12、私钥和密码不会进入日志、缓存或 Release asset

## 3. 增加发布身份门禁

- [x] 3.1 使用受保护的证书指纹 Secret 作为基准，并增加最终 App 的 `codesign --verify --deep --strict`、Identifier 与 requirement 校验
- [x] 3.2 解包 updater `.app.tar.gz` 并验证其 App 与 DMG 内 App 使用同一发布身份，同时保留现有 Minisign 验证
- [x] 3.3 通过必填 Secret 守卫、ad-hoc 负向校验和三份产物 requirement 比较，确认错误状态会阻止 finalize 发布

## 4. 跨版本权限验收

- [ ] 4.1 在无 ClipClop 历史记录的测试 Mac 上安装第一个固定身份构建，手动授予辅助功能权限并重启，确认自动粘贴与原生状态检测均为可用
- [ ] 4.2 安装使用同一证书构建的下一版本，确认无需重置 TCC 或重新授权即可继续自动粘贴
- [x] 4.3 记录自签名仍会触发 Gatekeeper/Chrome 警告、证书不可自动轮换及未来 Developer ID 迁移会要求一次重新授权

## 5. 最终验证

- [x] 5.1 运行 `openspec validate stabilize-macos-code-identity --strict` 并确认提案、设计、规范和任务全部通过
- [x] 5.2 运行现有前端、Rust 与 release workflow 静态检查，确认 Windows 构建和 Tauri updater 签名流程没有回归
