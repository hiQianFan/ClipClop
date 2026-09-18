# Distribution

`clipclop.io` serves manual downloads and Tauri updates from the
`clipclop-releases` R2 bucket. The website repository owns the public Worker,
domain routes and R2 read binding; this repository owns release production and
R2 writes.

## Public endpoints

- `https://clipclop.io/download/macos` redirects to the latest DMG.
- `https://clipclop.io/download/windows` redirects to the latest NSIS installer.
- `https://clipclop.io/latest.json` is the stable Tauri updater endpoint.
- `https://clipclop.io/releases.json` is the public release-history feed for App settings and the website changelog.
- `https://clipclop.io/releases/v<version>/` contains immutable, versioned release files.

The release workflow uploads every artifact before replacing `downloads.json`
and `latest.json`, then publishes the GitHub release and refreshes
`releases.json`. Versioned objects are cached for one year as immutable;
metadata and redirects use `no-cache`; `releases.json` uses a five-minute
public cache.

## Repository boundary

The website repository deploys the Worker and serves R2 objects through a
read-only binding. This repository does not deploy a public Worker. Release
uploads use the bucket-scoped
`CLOUDFLARE_RELEASES_R2_ACCESS_KEY_ID` and
`CLOUDFLARE_RELEASES_R2_SECRET_ACCESS_KEY` secrets in `production-release`.
The release workflow uploads and verifies versioned files before replacing
`downloads.json` and `latest.json`; keep this ordering atomic. `releases.json`
is generated from public GitHub Releases after publication, and a separate
scheduled/manual sync workflow refreshes it after release-note edits.

`clipclop.mapin.net` is not a supported compatibility endpoint and has no
redirect or proxy contract.

## macOS release identity

Production macOS builds use the fixed self-signed code-signing identity
`ClipClop Release` with bundle ID `io.clipclop.desktop`. This stabilizes the app's
code identity between releases, but it is not an Apple Developer ID and does
not remove Gatekeeper or browser download warnings.

Keep the encrypted certificate backup outside the repository at
`ClipClop/Credentials/macOS/ClipClop-Release.p12`. The `production-release`
environment must contain:

- `MACOS_CERTIFICATE_BASE64`: Base64-encoded contents of the P12 file.
- `MACOS_CERTIFICATE_PASSWORD`: the P12 export password.
- `MACOS_CERTIFICATE_SHA256`: the leaf certificate's SHA-256 fingerprint.

The workflow imports the certificate into a temporary keychain, signs through
Tauri using `ClipClop Release`, verifies the app, DMG and updater archive, then
deletes the temporary keychain. Missing or mismatched credentials stop the
macOS release instead of falling back to ad-hoc signing. The separate
`TAURI_SIGNING_PRIVATE_KEY` secrets continue to sign updater artifacts.
