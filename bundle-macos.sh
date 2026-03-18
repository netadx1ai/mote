#!/usr/bin/env bash
set -euo pipefail

APP_NAME="Mote"
BUNDLE_ID="com.moteapp.app"
VERSION="0.8.0"
BINARY_NAME="mote"
INSTALL_DIR="/Applications"

APP_BUNDLE="${INSTALL_DIR}/${APP_NAME}.app"
CONTENTS="${APP_BUNDLE}/Contents"
MACOS_DIR="${CONTENTS}/MacOS"
RESOURCES_DIR="${CONTENTS}/Resources"

echo "==> Building release binary..."
cargo build --release

echo "==> Creating ${APP_NAME}.app bundle..."
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

# Copy binary
cp "target/release/${BINARY_NAME}" "${MACOS_DIR}/${BINARY_NAME}"
chmod +x "${MACOS_DIR}/${BINARY_NAME}"

# Copy icon if available
ICON_SRC="assets/AppIcon.icns"
if [ -f "${ICON_SRC}" ]; then
    cp "${ICON_SRC}" "${RESOURCES_DIR}/AppIcon.icns"
elif [ -f "${RESOURCES_DIR}/AppIcon.icns" ]; then
    echo "    (keeping existing icon)"
else
    echo "    (no icon found at ${ICON_SRC}, skipping)"
fi

# Write Info.plist
cat > "${CONTENTS}/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>${BINARY_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
PLIST

# Ad-hoc code sign
echo "==> Signing ${APP_NAME}.app..."
codesign -fs - "${APP_BUNDLE}"

BINARY_SIZE=$(du -h "${MACOS_DIR}/${BINARY_NAME}" | cut -f1)
echo "==> Done! ${APP_BUNDLE} (${BINARY_SIZE})"
