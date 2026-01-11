#!/bin/bash
# =============================================================================
# APRK OS - Disk Image Creation Script (Absolute Isolation)
# =============================================================================
set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SOURCE_DIR="$PROJECT_ROOT/disk_root"
DEST_IMG="$PROJECT_ROOT/disk.img"

# Safe names without spaces
SAFE_TMP="/tmp/aprk_build_$(date +%s)"
SAFE_SRC="$SAFE_TMP/src"
SAFE_DMG="$SAFE_TMP/disk.dmg"
SAFE_RAW="$SAFE_TMP/disk.raw"

mkdir -p "$SAFE_SRC"

echo "Syncing files to safe location..."
cp -r "$SOURCE_DIR/" "$SAFE_SRC/"

echo "Creating raw FAT32 volume in isolated environment..."
hdiutil create -fs MS-DOS -volname "APRK" -srcfolder "$SAFE_SRC" -layout NONE -ov "$SAFE_DMG" > /dev/null

echo "Converting to raw format..."
hdiutil convert "$SAFE_DMG" -format UDTO -o "$SAFE_RAW" -ov > /dev/null

echo "Finalizing..."
cp "$SAFE_RAW.cdr" "$DEST_IMG"

echo "Cleaning up..."
rm -rf "$SAFE_TMP"

echo "Success: Disk image created at $DEST_IMG"
