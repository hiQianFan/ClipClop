#!/usr/bin/env bash
set -euo pipefail

bundle_dir="${1:?bundle directory is required}"
expected_fingerprint="${2:?certificate SHA-256 fingerprint is required}"
expected_identifier="io.clipclop.desktop"
work_dir="$(mktemp -d)"
mounted_dmg=""
trap '[[ -z "$mounted_dmg" ]] || hdiutil detach "$mounted_dmg" -quiet; rm -rf "$work_dir"' EXIT

expected_fingerprint="$(tr '[:lower:]' '[:upper:]' <<<"${expected_fingerprint//:/}")"
[[ "$expected_fingerprint" =~ ^[0-9A-F]{64}$ ]]

verify_app() {
  local app="$1" details fingerprint requirement
  codesign --verify --deep --strict --verbose=2 "$app" || return 1
  details="$(codesign -dv --verbose=4 "$app" 2>&1)" || return 1
  grep -Fqx "Identifier=$expected_identifier" <<<"$details" || return 1
  if grep -Fqx "Signature=adhoc" <<<"$details"; then return 1; fi
  codesign -d --extract-certificates="$work_dir/cert-" "$app" || return 1
  test -f "$work_dir/cert-0" || return 1
  fingerprint="$(openssl x509 -inform DER -in "$work_dir/cert-0" -noout -fingerprint -sha256 | cut -d= -f2 | tr -d ':' | tr '[:lower:]' '[:upper:]')"
  test "$fingerprint" = "$expected_fingerprint" || return 1
  requirement="$(codesign -dr - "$app" 2>&1 | sed -n 's/^designated => //p')" || return 1
  test -n "$requirement" || return 1
  printf '%s\n' "$requirement"
}

app="$bundle_dir/macos/ClipClop.app"
archive="$bundle_dir/macos/ClipClop.app.tar.gz"
dmg="$(find "$bundle_dir/dmg" -maxdepth 1 -name '*.dmg' -print -quit)"
test -d "$app"
test -f "$archive"
test -n "$dmg"

app_requirement="$(verify_app "$app")"
tar -xzf "$archive" -C "$work_dir"
archive_requirement="$(verify_app "$work_dir/ClipClop.app")"

mounted_dmg="$work_dir/dmg"
mkdir "$mounted_dmg"
hdiutil attach "$dmg" -readonly -nobrowse -mountpoint "$mounted_dmg" -quiet
dmg_requirement="$(verify_app "$mounted_dmg/ClipClop.app")"

test -n "$app_requirement"
test "$archive_requirement" = "$app_requirement"
test "$dmg_requirement" = "$app_requirement"
printf '%s\n' "Verified macOS release identity: $expected_identifier"
