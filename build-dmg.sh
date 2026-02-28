#!/bin/bash

# Builds a custom DMG containing the app + a double-clickable Install.command.
# Files inside a mounted DMG are not individually quarantined by macOS,
# so Install.command can run directly without Terminal or xattr workarounds.
#
# Usage: bash build-dmg.sh
# Run after: npm run tauri build

set -e

APP_NAME="ChatGPT to Claude"
VERSION="0.1.0"
APP_BUNDLE="src-tauri/target/release/bundle/macos/$APP_NAME.app"
OUT_DMG="src-tauri/target/release/bundle/dmg/${APP_NAME}_${VERSION}_aarch64.dmg"
STAGING="/tmp/chatgpt-to-claude-dmg"

echo ""
echo "  Building custom DMG..."
echo ""

# Verify app bundle exists
if [ ! -d "$APP_BUNDLE" ]; then
    echo "  ✗ App bundle not found. Run 'npm run tauri build' first."
    exit 1
fi

# Clean up any previous staging
rm -rf "$STAGING"
mkdir -p "$STAGING"

# Copy app bundle
echo "  → Copying app bundle..."
cp -R "$APP_BUNDLE" "$STAGING/"

# Copy install script and make executable
echo "  → Adding Install.command..."
cp Install.command "$STAGING/"
chmod +x "$STAGING/Install.command"

# Add Applications symlink for traditional drag-install option
ln -s /Applications "$STAGING/Applications"

# Remove old DMG if it exists
rm -f "$OUT_DMG"

# Create the DMG
echo "  → Creating DMG..."
hdiutil create \
    -volname "$APP_NAME" \
    -srcfolder "$STAGING" \
    -ov \
    -format UDZO \
    "$OUT_DMG"

# Clean up staging
rm -rf "$STAGING"

echo ""
echo "  ✓ DMG ready at: $OUT_DMG"
echo ""
