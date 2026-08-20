# ClipClop Privacy

[简体中文](PRIVACY.zh-CN.md) | English

ClipClop keeps clipboard history on the current device. It has no account, cloud sync, telemetry, advertising, or content-analysis service, and copied URLs do not trigger background content requests.

## Data stored locally

ClipClop may store supported clipboard contents, source-application metadata, file-path references, and settings. It does not automatically identify or filter sensitive content. File entries reference their original paths. File preview is off by default; while it is off, ClipClop shows stored names, paths, and source metadata without reading the original files. If you enable file preview, an explicit preview may read the selected source file to obtain its size, create a thumbnail, or show it with Quick Look. ClipClop does not copy, move, or upload the original file.

The local database is protected by the current operating-system account and disk encryption such as FileVault or BitLocker. ClipClop does not currently add application-level database encryption.

## Network access

When automatic update checks are enabled, ClipClop contacts `clipclop.mapin.net` (hosted on Cloudflare R2) at most once every 24 hours to compare versions, retrieve update information, and download an update you choose to install. The release-notes view also contacts GitHub Releases. These requests do not include clipboard contents, history, file paths, or source-application metadata. Automatic checks can be disabled in Settings.

## Your controls

You can delete individual entries, clear all history, limit history by age and item count, or quit ClipClop to stop capture. When recently used items are moved to the top, cleanup uses their last-used time. Uninstall behavior varies by operating system and installer; to remove all local data, quit ClipClop and delete the application data directory for `com.clipclop.desktop`.

## System permissions

On macOS, optional automatic paste uses Accessibility permission only to send the paste shortcut after you choose an item. If permission is denied, ClipClop still copies the item and you can paste manually. Optional file preview has a separate Full Disk Access shortcut in Quick Start and Settings; macOS requires you to grant this access manually, then confirm in ClipClop before it reads original files. You can revoke the system permission or disable file preview at any time. If you skip or disable it, file history remains available as stored basic information without original-file access. The first-run quick start uses only built-in examples and does not display your clipboard history.

This notice must be updated before adding telemetry, cloud sync, or any new network feature that processes clipboard data.
