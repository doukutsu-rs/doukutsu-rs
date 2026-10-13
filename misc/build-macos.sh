#!/bin/sh
# Builds a release doukutsu-rs.app: cargo-bundle, Liquid Glass icon from the Icon Composer project, ad-hoc signing.
# Requires cargo-bundle and Xcode 26 or newer (for compiling the .icon).
# Usage: misc/build-macos.sh [target-triple]
# Prints the path of the finished bundle on success.
set -eu

cd "$(dirname "$0")/.."

TARGET="${1:-}"
ICON_NAME="doukutsu-rs"

if [ -n "$TARGET" ]; then
  CARGO_INCREMENTAL=1 cargo bundle --release --format osx --target "$TARGET" >&2
  APP="target/$TARGET/release/bundle/osx/doukutsu-rs.app"
else
  CARGO_INCREMENTAL=1 cargo bundle --release --format osx >&2
  APP="target/release/bundle/osx/doukutsu-rs.app"
fi

MIN_VERSION="$(plutil -extract LSMinimumSystemVersion raw "$APP/Contents/Info.plist" 2>/dev/null || echo 10.15)"

OUT="$(mktemp -d)"
trap 'rm -rf "$OUT"' EXIT

# Produces Assets.car (used by macOS 26+) and a pre-rendered .icns fallback for older systems.
xcrun actool "res/$ICON_NAME.icon" \
  --compile "$OUT" \
  --output-partial-info-plist "$OUT/partial.plist" \
  --app-icon "$ICON_NAME" \
  --include-all-app-icons \
  --platform macosx \
  --target-device mac \
  --minimum-deployment-target "$MIN_VERSION" \
  --errors --warnings --output-format human-readable-text >&2

cp "$OUT/Assets.car" "$APP/Contents/Resources/"
cp "$OUT/$ICON_NAME.icns" "$APP/Contents/Resources/"

# Replaces the icon keys written by cargo-bundle with the ones actool expects.
plutil -replace CFBundleIconFile -string "$ICON_NAME" "$APP/Contents/Info.plist"
plutil -replace CFBundleIconName -string "$ICON_NAME" "$APP/Contents/Info.plist"

codesign -s - -f "$APP" >&2

echo "$APP"
