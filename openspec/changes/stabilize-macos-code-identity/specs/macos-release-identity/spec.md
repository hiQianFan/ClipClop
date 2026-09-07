## Purpose

为 ClipClop 的 macOS 正式产物提供跨版本稳定、可审计的代码身份，使系统权限能够识别同一应用的后续版本，并在身份意外变化时阻止发布。

## ADDED Requirements

### Requirement: 正式 macOS 产物必须具有稳定代码身份

ClipClop 的所有正式 macOS App、DMG 内 App 和 updater App 包 MUST 使用同一长期签名身份，且 Bundle ID 与签名标识 MUST 对应 `io.clipclop.desktop`。正式产物 MUST NOT 使用 unsigned 或 ad-hoc 代码身份。

#### Scenario: 构建新的正式版本

- **WHEN** 发布流水线生成 macOS 正式产物
- **THEN** 最终 App 的代码签名有效且不是 ad-hoc
- **AND** DMG 与 updater 包内 App 使用相同签名身份
- **AND** designated requirement 满足已登记的发布身份基准

#### Scenario: 后续版本复用身份

- **WHEN** 两个不同版本均作为正式 macOS 版本构建
- **THEN** 两个版本的可执行内容和 CDHash 可以不同
- **AND** 两个版本 MUST 满足同一发布身份 designated requirement

### Requirement: 签名材料必须受到保护

代码签名私钥和密码 MUST 只存在于受控离线备份及受保护的发布环境中，MUST NOT 出现在源码、构建日志、缓存或公开发布附件中。

#### Scenario: CI 执行 macOS 签名

- **WHEN** macOS 发布 job 需要使用签名私钥
- **THEN** job 从受保护 Secret 临时导入签名身份
- **AND** job 完成后删除临时 Keychain
- **AND** 日志不包含可恢复的私钥或密码

#### Scenario: 非发布构建运行

- **WHEN** pull request、普通质量检查或 Windows 构建运行
- **THEN** 它们不能读取 macOS 正式签名私钥

### Requirement: 身份异常必须阻止发布

发布流水线 MUST 在正式产物缺少签名、退回 ad-hoc、签名损坏、标识不符或 designated requirement 漂移时失败，并 MUST NOT 发布对应 Release。

#### Scenario: 发布 Secret 缺失

- **WHEN** macOS 正式构建无法加载预期签名身份
- **THEN** 构建在上传或发布产物前失败
- **AND** 不得自动创建替代证书或降级为 ad-hoc

#### Scenario: 最终产物身份不匹配

- **WHEN** 构建后的 App 不满足已登记的发布身份基准
- **THEN** 发布门禁失败
- **AND** 草稿 Release 保持未发布状态

### Requirement: 签名身份轮换必须显式进行

签名证书或 designated requirement 的任何变化 MUST 作为身份迁移处理，MUST NOT 在普通版本发布中静默发生。

#### Scenario: 证书需要更换

- **WHEN** 原证书丢失、泄露、到期或迁移到 Developer ID
- **THEN** 发布者必须明确批准身份轮换
- **AND** 发行说明必须告知 macOS 用户可能需要重新授予系统权限

### Requirement: Bundle ID 必须保持稳定

首个稳定身份版本及其所有后续正式版本 MUST 使用 `io.clipclop.desktop`。Bundle ID 的变化 MUST 被视为身份迁移，MUST NOT 在普通版本中静默发生。

#### Scenario: 发布后续普通版本

- **WHEN** 发布流水线构建首个稳定身份版本之后的普通更新
- **THEN** App、签名检查和相关运行时配置继续使用 `io.clipclop.desktop`

#### Scenario: Bundle ID 意外变化

- **WHEN** 最终产物的 Bundle ID 不再是 `io.clipclop.desktop`
- **THEN** 发布门禁必须失败

### Requirement: macOS 代码签名与 updater 签名必须相互独立

发布流程 MUST 同时验证 macOS 代码签名和 Tauri updater 签名，任一签名成功 MUST NOT 代替另一项验证。

#### Scenario: updater 签名有效但代码身份无效

- **WHEN** updater 包具有有效 Minisign 签名但内部 App 不满足 macOS 代码身份要求
- **THEN** 发布必须失败

#### Scenario: 代码身份有效但 updater 签名缺失

- **WHEN** macOS App 代码签名有效但 updater 包缺少有效 Minisign 签名
- **THEN** 发布必须失败

