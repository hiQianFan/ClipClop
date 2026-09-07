## Why

当前 macOS 发布包使用 ad-hoc 签名，代码身份绑定每次构建变化的 CDHash，导致辅助功能权限可能在安装或升级后仍显示已开启但当前进程无法使用。ClipClop 尚无存量安装用户，现在是切换到稳定发布身份、避免形成迁移负担的最佳时机。

## What Changes

- 为 macOS 正式发布建立一张长期固定的自签名 Code Signing 证书，并在所有架构和后续版本中复用同一身份。
- 在发布流水线中从受保护 Secret 导入证书到临时 Keychain，仅 macOS job 可访问私钥。
- 在首次稳定身份发布前，将产品 Bundle ID 统一为 `io.clipclop.desktop`；发布后将其视为不可随普通版本变更的持久身份。
- 使用固定 Bundle ID `io.clipclop.desktop` 对完整 App bundle 签名，并验证签名有效、不是 ad-hoc、代码身份与 designated requirement 稳定。
- 在发布前比较基准 designated requirement；身份意外变化时阻止发布，避免静默使用户权限失效。
- 记录证书备份、恢复和轮换规则；证书或私钥变化视为需要用户重新授权的破坏性发布事件。
- 保留现有 Tauri updater Minisign 签名；它与 macOS 代码签名承担不同职责。
- 不增加旧版本 TCC 修复流程、静默运行 `tccutil` 或权限数据库操作。

## Capabilities

### New Capabilities

- `macos-release-identity`: macOS 正式构建的稳定代码身份、密钥保护、发布门禁及权限连续性契约。

### Modified Capabilities

无。

## Impact

- 发布流水线：`.github/workflows/bundle.yml` 的 macOS 构建、临时 Keychain 和产物验证步骤。
- 应用身份与数据路径：将当前 `com.clipclop.desktop` 统一迁移为 `io.clipclop.desktop`；由于尚无存量用户，不增加旧数据迁移。
- macOS 构建配置：`src-tauri/tauri.conf.json` 或发布专用配置中的签名身份。
- GitHub `production-release` Environment：新增证书与密码 Secret。
- 运维：需要离线保存同一张证书及私钥；丢失或轮换会改变代码身份。
- 用户体验：新用户仍需手动允许辅助功能权限，并仍会看到 Gatekeeper/Chrome 对未公证软件的警告。
