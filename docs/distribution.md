# ClipClop 构建与分发

## 本地质量检查

```bash
pnpm install --frozen-lockfile
pnpm check
pnpm build
cargo fmt --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## macOS

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin --bundles dmg
```

产物位于 `src-tauri/target/universal-apple-darwin/release/bundle/`。面向用户只发布 Universal `.dmg`；updater 使用同一次构建产生的 `.app.tar.gz` 和 `.sig`。当前开源预览版未配置 Apple Developer ID 和公证，用户首次打开时可能需要在系统“隐私与安全性”中手动允许。

如果本机 Xcode beta 导致 Tauri 的 Finder 布局脚本失败，可从已生成的 `.app` 创建标准只读测试镜像：

```bash
mkdir -p dist
hdiutil create -volname ClipClop \
  -srcfolder src-tauri/target/release/bundle/macos/ClipClop.app \
  -ov -format UDZO dist/ClipClop_0.1.0_aarch64.dmg
hdiutil verify dist/ClipClop_0.1.0_aarch64.dmg
```

该回退仅用于本地测试；正式发布仍使用稳定 Xcode、签名、公证和官方 bundler。

## Windows

建议在 Windows 实机或 GitHub Actions `windows-latest` 构建：

```powershell
pnpm tauri build --target x86_64-pc-windows-msvc --bundles nsis
```

面向个人用户只发布 NSIS setup `.exe`，不生成 MSI。当前预览版未配置 Authenticode，Windows 可能显示未知发布者或 SmartScreen 提示。

## 自动更新签名

`src-tauri/tauri.conf.json` 只包含 updater 公钥。私钥不得进入仓库，本地维护副本位于 `~/.tauri/clipclop.key`，CI 只通过 `production-release` Environment 注入：

```text
TAURI_SIGNING_PRIVATE_KEY
TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

Tauri updater 签名用于验证更新来源，不等同于 Apple/Windows 平台代码签名。

## CI

- `quality.yml` 在 macOS 与 Windows 执行前后端检查和测试。
- `bundle.yml` 可手动运行，或在推送语义化 `vX.Y.Z` tag 时构建 Universal DMG 和 Windows x64 NSIS，生成 updater artifacts/`latest.json` 并创建 Draft GitHub Release。
- 签名密钥只能通过 CI secrets 注入，禁止提交到仓库。
- Draft Release 先检查安装包、签名文件和 `latest.json`；确认无误后由维护者发布为普通 Release（不要标记为 prerelease，否则 `/releases/latest` 无法发现它）。旧版本自动升级需要在 Release 发布后完成验证，如有问题立即撤回 Release 并修复。
