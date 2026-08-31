# Distribution

`clipclop.mapin.net` serves both manual downloads and Tauri updates from the
`clipclop-releases` R2 bucket.

## Public endpoints

- `https://clipclop.mapin.net/download/macos` redirects to the latest DMG.
- `https://clipclop.mapin.net/download/windows` redirects to the latest NSIS installer.
- `https://clipclop.mapin.net/latest.json` remains the stable Tauri updater endpoint.
- `/releases/v<version>/` contains immutable, versioned release files.

The release workflow uploads every artifact before replacing `downloads.json`
and `latest.json`. Versioned objects are cached for one year as immutable;
metadata and redirects use `no-cache`.

## One-time Cloudflare setup

1. Keep `clipclop.mapin.net` connected to the `clipclop-releases` R2 bucket.
2. Create an **Edit Cloudflare Workers** API token, scoped to the ClipClop
   account. CI only needs `Workers Scripts Write` because the stable
   `clipclop.mapin.net/download/*` route is provisioned once, outside CI.
3. Save it as the `CLOUDFLARE_DOWNLOAD_WORKER_API_TOKEN` secret in the
   `production-downloads` GitHub environment.
4. Save `CLOUDFLARE_ACCOUNT_ID` as a repository variable, then run the
   **Deploy Download Worker** workflow once.

Release uploads use the bucket-scoped
`CLOUDFLARE_RELEASES_R2_ACCESS_KEY_ID` and
`CLOUDFLARE_RELEASES_R2_SECRET_ACCESS_KEY` secrets in `production-release`.
Run the Worker workflow again only when files under `cloudflare/` change.
