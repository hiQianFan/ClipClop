#!/usr/bin/env bash

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tray_root="$repo_root/src-tauri/icons/tray"
source_svg="$tray_root/source/tray-hooves.svg"

for required_tool in magick; do
  if ! command -v "$required_tool" >/dev/null 2>&1; then
    echo "Missing required tool: $required_tool" >&2
    exit 1
  fi
done

mkdir -p "$tray_root/macos" "$tray_root/windows"

magick -background none -density 384 "$source_svg" \
  -resize 36x36 "$tray_root/macos/trayTemplate@2x.png"

magick -background none -density 384 "$source_svg" \
  -resize 32x32 "$tray_root/windows/tray-light-32.png"
magick "$tray_root/windows/tray-light-32.png" \
  -channel RGB -fill white -colorize 100 "$tray_root/windows/tray-dark-32.png"

echo "Generated tray assets from $source_svg"
