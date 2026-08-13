#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tray_root="$repo_root/src-tauri/icons/tray"
macos_source="$tray_root/source/tray-hooves.svg"
windows_source="$tray_root/source/tray-cow-horse.png"

for required_tool in magick; do
  if ! command -v "$required_tool" >/dev/null 2>&1; then
    echo "Missing required tool: $required_tool" >&2
    exit 1
  fi
done

mkdir -p "$tray_root/macos" "$tray_root/windows"

magick -background none -density 384 "$macos_source" \
  -resize 36x36 "$tray_root/macos/trayTemplate@2x.png"

magick "$windows_source" -resize 30x30 -gravity center -background none \
  -extent 32x32 "$tray_root/windows/tray-32.png"

echo "Generated macOS and Windows tray assets"
