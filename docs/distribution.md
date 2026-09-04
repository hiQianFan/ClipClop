# Distribution

`clipclop.io` serves manual downloads and Tauri updates from the
`clipclop-releases` R2 bucket. The website repository owns the public Worker,
domain routes and R2 read binding; this repository owns release production and
R2 writes.

## Public endpoints

- `https://clipclop.io/download/macos` redirects to the latest DMG.
- `https://clipclop.io/download/windows` redirects to the latest NSIS installer.
- `https://clipclop.io/latest.json` is the stable Tauri updater endpoint.
- `https://clipclop.io/releases/v<version>/` contains immutable, versioned release files.

The release workflow uploads every artifact before replacing `downloads.json`
and `latest.json`. Versioned objects are cached for one year as immutable;
metadata and redirects use `no-cache`.

## Repository boundary

The website repository deploys the Worker and serves R2 objects through a
read-only binding. This repository does not deploy a public Worker. Release
uploads use the bucket-scoped
`CLOUDFLARE_RELEASES_R2_ACCESS_KEY_ID` and
`CLOUDFLARE_RELEASES_R2_SECRET_ACCESS_KEY` secrets in `production-release`.
The release workflow uploads and verifies versioned files before replacing
`downloads.json` and `latest.json`; keep this ordering atomic.

`clipclop.mapin.net` is not a supported compatibility endpoint and has no
redirect or proxy contract.
