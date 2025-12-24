#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "Building PP Killer AppImage..."
echo "Project root: $PROJECT_ROOT"

# Check if pnpm is installed
if ! command -v pnpm &> /dev/null; then
    echo "pnpm is not installed. Installing..."
    npm install -g pnpm
fi

# Install dependencies
echo "Installing dependencies..."
pnpm install

# Build Tauri application
echo "Building Tauri application..."
pnpm tauri build || {
    echo "Tauri build failed, but AppDir might be created. Checking..."
}

# Check if AppImage was created successfully
APPIMAGE=$(find ~/.cargo/target_global/release/bundle/appimage -name "PP Killer_*.AppImage" -o -name "PP-Killer_*.AppImage" 2>/dev/null | head -n 1)

# If AppImage not found, try to create it from AppDir using appimagetool
if [ -z "$APPIMAGE" ]; then
    APPDIR=$(find ~/.cargo/target_global/release/bundle/appimage -name "PP Killer.AppDir" -o -name "PP-Killer.AppDir" -o -name "PortKiller.AppDir" -type d 2>/dev/null | head -n 1)
    
    if [ -n "$APPDIR" ]; then
        echo "AppImage not found, but AppDir exists. Creating AppImage with appimagetool..."
        
        # Download appimagetool if not available
        if [ ! -f "/tmp/appimagetool.AppImage" ]; then
            echo "Downloading appimagetool..."
            wget -q https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage -O /tmp/appimagetool.AppImage
            chmod +x /tmp/appimagetool.AppImage
        fi
        
        # Ensure frontend files are copied to AppDir
        echo "Checking if frontend files are in AppDir..."
        if [ ! -f "$APPDIR/usr/lib/app/resources/index.html" ] && [ -d "$PROJECT_ROOT/ui" ]; then
            echo "Frontend files not found in AppDir. Copying them..."
            mkdir -p "$APPDIR/usr/lib/app/resources"
            cp -r "$PROJECT_ROOT/ui"/* "$APPDIR/usr/lib/app/resources/"
            echo "Frontend files copied to AppDir"
        fi
        
        # Ensure app.png exists in AppDir root (required by appimagetool)
        if [ ! -f "$APPDIR/app.png" ]; then
            if [ -f "$APPDIR/usr/share/icons/hicolor/128x128/apps/app.png" ]; then
                cp "$APPDIR/usr/share/icons/hicolor/128x128/apps/app.png" "$APPDIR/app.png"
                echo "Copied app.png to AppDir root"
            fi
        fi
        
        # Create AppImage from AppDir
        OUTPUT_NAME="PP-Killer-x86_64.AppImage"
        # Use ARCH environment variable to ensure proper architecture
        ARCH=x86_64 /tmp/appimagetool.AppImage "$APPDIR" "$PROJECT_ROOT/$OUTPUT_NAME" 2>&1 | grep -v "app{.png,.svg,.xpm} defined"
        
        if [ -f "$PROJECT_ROOT/$OUTPUT_NAME" ]; then
            chmod +x "$PROJECT_ROOT/$OUTPUT_NAME"
            echo "AppImage created successfully using appimagetool!"
            APPIMAGE="$PROJECT_ROOT/$OUTPUT_NAME"
        else
            echo "Error: Failed to create AppImage with appimagetool"
            exit 1
        fi
    else
        echo "Error: Neither AppImage nor AppDir found"
        exit 1
    fi
else
    # Copy to project root with a clean name
    OUTPUT_NAME="PP-Killer-x86_64.AppImage"
    cp "$APPIMAGE" "$PROJECT_ROOT/$OUTPUT_NAME"
    chmod +x "$PROJECT_ROOT/$OUTPUT_NAME"
fi

echo "Build complete!"
echo "AppImage location: $PROJECT_ROOT/$OUTPUT_NAME"
echo "Size: $(du -h "$PROJECT_ROOT/$OUTPUT_NAME" | cut -f1)"

