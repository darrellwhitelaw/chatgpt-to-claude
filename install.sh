#!/bin/bash

set -e

APP_NAME="ChatGPT to Claude"
DMG_VOLUME="/Volumes/$APP_NAME"
INSTALL_PATH="/Applications/$APP_NAME.app"

echo ""
echo "  Installing $APP_NAME..."
echo ""

# Make sure the DMG is mounted
if [ ! -d "$DMG_VOLUME" ]; then
    echo "  ⚠️  Please open the .dmg file first, then run this script again."
    echo ""
    exit 1
fi

# Copy app to Applications (overwrite if already there)
echo "  → Copying to Applications..."
cp -R "$DMG_VOLUME/$APP_NAME.app" "/Applications/"

# Strip the quarantine flag so macOS doesn't block it
echo "  → Clearing macOS quarantine flag..."
xattr -cr "$INSTALL_PATH"

echo ""
echo "  ✓ Installed. Launching now..."
echo ""

open "$INSTALL_PATH"
