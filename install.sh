#!/bin/bash

# PortKiller Linux - Premium Tauri Edition Installer

echo "Installing PortKiller (Tauri Premium Edition)..."

# Build Tauri App
echo "Building the GUI interface..."
npx -y @tauri-apps/cli build

# Define potential binary locations
LOCATIONS=(
    "/home/vaha/.cargo/target_global/release/app"
    "./src-tauri/target/release/app"
    "./src-tauri/target/release/portkiller"
)

BINARY_PATH=""
for loc in "${LOCATIONS[@]}"; do
    if [ -f "$loc" ]; then
        BINARY_PATH="$loc"
        break
    fi
done

if [ -z "$BINARY_PATH" ]; then
    echo "Error: Build failed, binary not found in expected locations."
    exit 1
fi

echo "Installing binary from $BINARY_PATH to ~/.local/bin/portkiller"
mkdir -p ~/.local/bin
cp "$BINARY_PATH" ~/.local/bin/portkiller
chmod +x ~/.local/bin/portkiller

# Desktop Entry
mkdir -p ~/.local/share/applications/
cat <<EOF > ~/.local/share/applications/portkiller.desktop
[Desktop Entry]
Name=Port Killer
Comment=Premium Port Management Tool (Liquid Glass)
Exec=$HOME/.local/bin/portkiller
Icon=network-transmit-receive-symbolic
Type=Application
Categories=Development;System;
Terminal=false
Keywords=port;kill;process;network;
EOF

echo "--------------------------------------------------"
echo "Port Killer Premium (Liquid Glass) installed!"
echo "Launch it from your applications menu or terminal."
echo "--------------------------------------------------"
