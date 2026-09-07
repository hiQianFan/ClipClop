## Context

见 `proposal.md`。当前本地正式 bundle 的签名检查结果为 `Signature=adhoc`、`TeamIdentifier=not set`、`designated => cdhash ...`，并且签名层的 Identifier 没有稳定表现为产品最终身份 `io.clipclop.desktop`。Apple 的代码签名文档说明，系统依靠 designated requirement 判断新版代码是否仍属于同一应用；ad-hoc 身份只代表具体构建。项目当前由 GitHub Actions 生成 universal macOS App/DMG，同时使用独立的 Tauri updater Minisign 密钥保护自动更新包。

## Goals / Non-Goals

**Goals:**

- 在不购买 Apple Developer Program 的阶段，为所有正式 macOS 版本提供可重复、可验证的代码身份。
- 让证书私钥只存在于创建者的离线备份和受保护 CI 环境中。
- 在发布前发现 ad-hoc 回退、错误 Bundle ID、签名损坏或身份漂移。
- 保持当前 updater 签名和跨平台发布流程不变。

**Non-Goals:**

- 不声称自签名等同 Developer ID，也不提供 Apple 公证、App Store 或无警告安装。
- 不自动授予、重置或编辑 TCC 权限。
- 不为尚不存在的旧用户增加迁移或修复 UI。
- 不把证书、私钥、密码或其可恢复形式提交到仓库或上传为 Release asset。

## Decisions

### D1. 使用一张长期固定的自签名 Code Signing 身份

创建专用于 ClipClop 正式发布的自签名根证书，证书用途为 Code Signing，名称固定为 `ClipClop Release`。证书及私钥导出为带强密码的 PKCS#12 文件，所有正式 macOS 版本和 universal 架构复用该身份。

选择自签名证书而不是继续 ad-hoc，是因为证书公钥可成为跨版本稳定 requirement 的锚点；Apple 也明确区分了“稳定代码身份”和“受 Gatekeeper 信任”。选择它仅作为免费阶段的过渡，而不是公共分发的最终信任方案。

### D2. 私钥采用离线主备份加 GitHub Environment Secret

创建者需要准备：

1. `ClipClop-Release.p12` 及独立强密码；
2. 至少一份加密离线备份，不能只保存在 GitHub；
3. `production-release` Environment 中的 Base64 PKCS#12 Secret 和密码 Secret；
4. 一份不含私钥的证书指纹与 designated requirement 基准，允许提交仓库用于验证。

发布凭据统一使用以下命名，避免旧 Bundle ID 继续出现在运维资料中：

- macOS 证书身份：`ClipClop Release`；
- macOS 证书备份：`ClipClop-Release.p12`；
- updater 私钥备份：`ClipClop-Updater.key`；
- updater 公钥备份：`ClipClop-Updater.key.pub`；
- updater 密码条目：`io.clipclop.desktop.updater`；
- macOS 证书密码条目：`io.clipclop.desktop.release-signing`。

iCloud Drive 中的凭据统一放在 `ClipClop/Credentials/`，与未来其他重要项目资料分开管理：

```text
ClipClop/Credentials/
├── README.md
├── macOS/ClipClop-Release.p12
└── Updater/
    ├── ClipClop-Updater.key
    └── ClipClop-Updater.key.pub
```

P12 和 updater 私钥必须保持加密，密码只存密码管理器，不写入该目录。iCloud 同步副本不能作为唯一备份；还需保留一份不同位置的加密离线副本，防止同步删除、账号锁定或文件损坏。

CI 在 macOS runner 上创建随机密码的临时 Keychain、导入 PKCS#12、允许 `/usr/bin/codesign` 使用，并在 job 结束时删除 Keychain。日志不得输出私钥、P12 内容或密码。

### D3. 完整签名 bundle，并让产物自证身份

签名必须覆盖 universal App 的主可执行文件及全部嵌套代码，最终外层 App 的签名 Identifier 必须是 `io.clipclop.desktop`。具体由 Tauri 原生签名入口或一个最小发布脚本完成；实现阶段优先使用 Tauri 已支持的签名环境变量，只有无法保证完整签名顺序时才增加脚本。

每次发布必须对最终 `.app` 执行：

- `codesign --verify --deep --strict --verbose=2`；
- `codesign -dv --verbose=4`，确认不是 `adhoc` 且标识正确；
- `codesign -dr -`，确认 designated requirement 与仓库基准身份一致；
- 解包 updater `.app.tar.gz` 后再次验证，确保 DMG 与 updater 中承载的是同一身份。

不以 `spctl` 通过作为当前阶段门禁，因为自签名产物在默认 Gatekeeper 策略下仍会被拒绝；将来切换 Developer ID 后再加入公证和 Gatekeeper 验证。

### D4. 身份漂移必须阻止发布

证书过期、丢失、被替换、Secret 缺失、构建退回 ad-hoc 或 designated requirement 改变时，macOS job 必须失败，草稿 Release 不得发布。不能自动生成替代证书，因为这会把一次可见的发布失败变成所有用户权限静默失效。

证书轮换必须作为显式迁移发布处理，并在发行说明中告知用户需要重新授权。当前尚无存量用户，因此首次采用该身份不需要兼容旧 ad-hoc grant。

### D5. 与 updater 签名保持职责隔离

`TAURI_SIGNING_PRIVATE_KEY` 继续为 updater manifest/包提供完整性校验；PKCS#12 仅用于 macOS 代码身份。两个密钥不得复用，也不得因其中一个配置成功而跳过另一个的验证。

## Risks / Trade-offs

- **自签名不受 Gatekeeper 信任** → 保留当前安装说明；将来以 Developer ID 和公证替换。
- **部分未来 macOS 版本可能收紧 TCC 对自签名应用的接受** → 把真实跨版本权限测试列为发布验收，但不承诺超出 Apple 支持范围的永久兼容。
- **私钥丢失会迫使身份轮换** → 离线加密备份至少一份，恢复演练后再启用 CI。
- **私钥泄露无法像 Developer ID 一样由 Apple 撤销** → 限定 GitHub Environment、最小访问权限；泄露时停止发布并显式轮换。
- **切换到 Developer ID 时身份再次变化** → 作为未来一次性迁移处理，并提前通知重新授权。

## Migration Plan

1. 在首个稳定身份版本发布前，将应用 Bundle ID 及所有内部引用统一为 `io.clipclop.desktop`。
2. 在受控 Mac 上创建证书并导出加密 PKCS#12；记录证书 SHA-256 指纹。
3. 完成离线备份与恢复验证后，将 P12 和密码写入 GitHub `production-release` Environment Secrets。
4. 在未发布的测试版本接入 CI 签名和门禁，下载 DMG 与 updater 包并核对身份。
5. 在一台从未安装 ClipClop 的 Mac 上完成首次授权、退出重启和自动粘贴验证。
6. 安装使用同一证书签名的下一测试版本，确认辅助功能权限继续有效。
7. 验收通过后才发布首个固定身份版本；此后不得在普通发布中修改 Bundle ID。

回滚仅允许回滚发布流水线代码，不允许在正式版本中退回 ad-hoc 签名。若固定身份无法使用，应暂停 macOS 发布而不是生成新身份。
