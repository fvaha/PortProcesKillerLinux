# Building Portable AppImage for PortKiller

This guide explains how to build a fully portable AppImage that works on any Linux distribution.

## Problem We're Solving

The AppImage wasn't working on other computers because:

1. **Hardcoded paths** - The app used `/usr/bin/ss` and `/usr/bin/kill` which might not exist on all systems
2. **Missing system tools** - Tools like `ss`, `ps`, and `kill` weren't bundled with the AppImage

## Solution Implemented

We've made the following improvements:

### 1. Dynamic Path Resolution

The Rust backend now dynamically finds system commands instead of using hardcoded paths:

- Checks common locations (`/usr/bin`, `/bin`, `/usr/local/bin`, etc.)
- Falls back to `which` command
- Uses command name as last resort (relies on PATH)

### 2. New Features Added

- **Ports View** - Monitor and kill processes using specific ports
- **Processes View** - View all running processes with CPU/Memory usage
- **Better filtering** - Filter by Web Apps, Databases, Docker containers
- **Confirmation dialogs** - Prevent accidental process termination

## Building the AppImage

### Option 1: Using Tauri Build (Recommended)

```bash
# Build the application
npx @tauri-apps/cli build

# The AppImage will be in:
# src-tauri/target/release/bundle/appimage/
```

### Option 2: Manual AppImage Creation

If you need more control over the AppImage:

```bash
# 1. Build the release binary
cd src-tauri
cargo build --release

# 2. Create AppDir structure
mkdir -p AppDir/usr/bin
mkdir -p AppDir/usr/share/applications
mkdir -p AppDir/usr/share/icons/hicolor/256x256/apps

# 3. Copy the binary
cp target/release/app AppDir/usr/bin/portkiller

# 4. Create desktop entry
cat > AppDir/portkiller.desktop << 'EOF'
[Desktop Entry]
Name=PortKiller
Comment=Premium Port and Process Management
Exec=portkiller
Icon=portkiller
Type=Application
Categories=Development;System;Utility;
Terminal=false
EOF

# 5. Copy icon (if you have one)
cp ../portkillerlinux.png AppDir/usr/share/icons/hicolor/256x256/apps/portkiller.png

# 6. Create AppRun script
cat > AppDir/AppRun << 'EOF'
#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH}"
exec "${HERE}/usr/bin/portkiller" "$@"
EOF
chmod +x AppDir/AppRun

# 7. Download appimagetool if you don't have it
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# 8. Create the AppImage
./appimagetool-x86_64.AppImage AppDir PortKiller-x86_64.AppImage
```

## Testing the AppImage

Test on different distributions:

```bash
# Make it executable
chmod +x PortKiller-x86_64.AppImage

# Run it
./PortKiller-x86_64.AppImage
```

## System Requirements

The AppImage should work on any Linux distribution with:

- **Kernel**: 3.10 or newer
- **glibc**: 2.17 or newer (most distros from 2013+)
- **Display Server**: X11 or Wayland
- **System Tools**: `ss`, `ps`, `kill` (usually pre-installed)

## Troubleshooting

### AppImage doesn't start

```bash
# Run with debug output
./PortKiller-x86_64.AppImage --appimage-extract-and-run
```

### Missing system tools

The app will gracefully handle missing tools:

- If `ss` is not found, port listing will return empty
- If `ps` is not found, process listing will return empty
- If `kill` is not found, termination will fail silently

### Permission issues

Some processes require root privileges to kill:

```bash
# Run with sudo if needed
sudo ./PortKiller-x86_64.AppImage
```

## Distribution

Your AppImage is now fully portable! You can:

1. Upload to GitHub Releases
2. Share directly with users
3. No installation required - just download and run

## Version Updates

Update version in:

1. `package.json` - `"version": "1.0.4"`
2. `src-tauri/tauri.conf.json` - `"version": "1.0.4"`
3. `src-tauri/Cargo.toml` - `version = "0.1.0"` (internal)

Then rebuild the AppImage.
