#!/bin/bash

APP_NAME="ChatGPT to Claude"
DMG_VOLUME="/Volumes/$APP_NAME"
INSTALL_PATH="/Applications/$APP_NAME.app"

echo ""
echo "  Installing $APP_NAME..."
echo ""

# Copy to Applications
echo "  → Copying to Applications..."
cp -R "$DMG_VOLUME/$APP_NAME.app" "/Applications/"

# Strip the quarantine flag
echo "  → Clearing macOS quarantine flag..."
xattr -cr "$INSTALL_PATH"

echo ""
echo "  ✓ Done! Launching $APP_NAME..."
echo ""

open "$INSTALL_PATH"

# Close Terminal window
sleep 1
osascript -e 'tell application "Terminal" to close front window'
