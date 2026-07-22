# ClipClop Privacy

[简体中文](PRIVACY.zh-CN.md) | English

ClipClop keeps clipboard history on the current device. It has no account, cloud sync, telemetry, advertising, or content-analysis service, and copied URLs do not trigger background content requests.

## Data stored locally

ClipClop may store supported clipboard contents, source-application metadata, file-path references, and settings. It does not automatically identify or filter sensitive content. File entries reference their original paths; explicitly selecting or previewing one may read the source file to obtain its size or create a thumbnail, but ClipClop does not copy, move, or upload it.

The local database is protected by the current operating-system account and disk encryption such as FileVault or BitLocker. ClipClop does not currently add application-level database encryption.

## Network access

When automatic update checks are enabled, ClipClop contacts GitHub Releases at most once every 24 hours to compare versions and retrieve update information. These requests do not include clipboard contents, history, file paths, or source-application metadata. Automatic checks can be disabled in Settings.

## Your controls

You can delete individual entries, clear all history, choose a retention period, or quit ClipClop to stop capture. Uninstall behavior varies by operating system and installer; to remove all local data, quit ClipClop and delete the application data directory for `com.clipclop.desktop`.

This notice must be updated before adding telemetry, cloud sync, or any new network feature that processes clipboard data.
