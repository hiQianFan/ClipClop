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
pnpm tauri build --bundles app,dmg
```

产物位于 `src-tauri/target/release/bundle/`。对外分发需要 Apple Developer ID 签名与公证；未配置证书时只能用于本机或测试环境验证。

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
pnpm tauri build --bundles msi,nsis
```

产物为 `.msi` 和 NSIS setup `.exe`。公开分发建议使用代码签名证书，减少 SmartScreen 警告。

## CI

- `quality.yml` 在 macOS 与 Windows 执行前后端检查和测试。
- `bundle.yml` 可手动运行，或在推送 `v*` tag 时生成双端安装包并上传为 Actions artifacts。
- 签名密钥只能通过 CI secrets 注入，禁止提交到仓库。
