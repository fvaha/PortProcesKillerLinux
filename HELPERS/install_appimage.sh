#!/bin/bash

# PortKiller Linux - Waybar Setup za AppImage
# Ažurira Waybar konfiguraciju da koristi portable AppImage

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

# Pronađi AppImage - koristi prvi koji se nađe ili argument
APPIMAGE=""
if [ -n "$1" ] && [ -f "$1" ]; then
    # Ako je dat argument, koristi ga kao putanju do AppImage-a
    APPIMAGE="$1"
elif [ -f "$PROJECT_ROOT/PP-Killer-x86_64.AppImage" ]; then
    APPIMAGE="$PROJECT_ROOT/PP-Killer-x86_64.AppImage"
elif [ -f "$PROJECT_ROOT/PortKiller-x86_64.AppImage" ]; then
    APPIMAGE="$PROJECT_ROOT/PortKiller-x86_64.AppImage"
else
    echo "Error: AppImage not found"
    echo ""
    echo "Usage: $0 [path-to-AppImage]"
    echo "  or place PP-Killer-x86_64.AppImage in project root"
    echo ""
    echo "Build AppImage first: ./HELPERS/build_appimage.sh"
    exit 1
fi

# Konvertuj u apsolutnu putanju
APPIMAGE=$(readlink -f "$APPIMAGE" 2>/dev/null || realpath "$APPIMAGE" 2>/dev/null || echo "$APPIMAGE")

if [ ! -f "$APPIMAGE" ]; then
    echo "Error: AppImage not found at: $APPIMAGE"
    exit 1
fi

echo "Using AppImage: $APPIMAGE"
echo "AppImage is portable - no installation needed!"

# Instaliraj AppImage u ~/.local/bin
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"

INSTALL_PATH="$INSTALL_DIR/ppkiller.AppImage"
echo "Installing AppImage to $INSTALL_PATH..."
cp "$APPIMAGE" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

# Kreiraj wrapper skript za lakše pozivanje
WRAPPER_PATH="$INSTALL_DIR/ppkiller"
cat > "$WRAPPER_PATH" << EOF
#!/bin/bash
exec "$INSTALL_PATH" "\$@"
EOF
chmod +x "$WRAPPER_PATH"

echo "AppImage installed successfully!"

# Ažuriraj Waybar konfiguraciju da koristi AppImage sa trenutne lokacije
WAYBAR_CONFIG="$HOME/.config/waybar/config"
if [ -f "$WAYBAR_CONFIG" ]; then
    echo ""
    echo "Updating Waybar configuration..."
    
    # Backup originalne konfiguracije
    cp "$WAYBAR_CONFIG" "$WAYBAR_CONFIG.backup.$(date +%Y%m%d_%H%M%S)"
    
    # Escape putanju za sed (zameni / sa \/)
    ESCAPED_PATH=$(echo "$APPIMAGE" | sed 's|/|\\/|g')
    
    # Zameni stare reference na portkiller sa novim AppImage putanjom
    sed -i "s|~/.local/bin/portkiller|$APPIMAGE|g" "$WAYBAR_CONFIG"
    sed -i "s|$HOME/.local/bin/portkiller|$APPIMAGE|g" "$WAYBAR_CONFIG"
    sed -i "s|/home/vaha/.local/bin/portkiller|$APPIMAGE|g" "$WAYBAR_CONFIG"
    
    # Ažuriraj modul naziv ako je potrebno (custom/portkiller -> custom/ppkiller)
    if grep -q '"custom/portkiller"' "$WAYBAR_CONFIG" || grep -q 'custom/portkiller' "$WAYBAR_CONFIG"; then
        # Zameni u modules-left/modules-right listi
        sed -i 's|"custom/portkiller"|"custom/ppkiller"|g' "$WAYBAR_CONFIG"
        sed -i "s|'custom/portkiller'|'custom/ppkiller'|g" "$WAYBAR_CONFIG"
        sed -i 's|custom/portkiller|custom/ppkiller|g' "$WAYBAR_CONFIG"
        
        # Ažuriraj i definiciju modula
        if grep -q '"custom/portkiller":' "$WAYBAR_CONFIG"; then
            sed -i 's|"custom/portkiller":|"custom/ppkiller":|g' "$WAYBAR_CONFIG"
        fi
    fi
    
    # Ažuriraj putanju u custom/ppkiller modulu ako već postoji
    if grep -q '"custom/ppkiller":' "$WAYBAR_CONFIG"; then
        # Zameni exec, on-click i on-click-right putanje sa apsolutnom putanjom
        sed -i "s|\"exec\": \".*waybar\"|\"exec\": \"$APPIMAGE waybar\"|g" "$WAYBAR_CONFIG"
        sed -i "s|\"on-click\": \".*menu\"|\"on-click\": \"$APPIMAGE menu\"|g" "$WAYBAR_CONFIG"
        sed -i "s|\"on-click-right\": \".*\"|\"on-click-right\": \"$APPIMAGE\"|g" "$WAYBAR_CONFIG"
    fi
    
    echo "Waybar configuration updated!"
    
    # Ažuriraj CSS fajl ako postoji
    WAYBAR_CSS="$HOME/.config/waybar/style.css"
    if [ -f "$WAYBAR_CSS" ]; then
        echo "Updating Waybar CSS..."
        # Backup CSS fajla
        cp "$WAYBAR_CSS" "$WAYBAR_CSS.backup.$(date +%Y%m%d_%H%M%S)"
        
        # Zameni #custom-portkiller sa #custom-ppkiller
        sed -i 's|#custom-portkiller|#custom-ppkiller|g' "$WAYBAR_CSS"
        
        echo "Waybar CSS updated!"
    fi
    
    echo ""
    echo "Restart Waybar to apply changes:"
    echo "  killall -SIGUSR2 waybar"
else
    echo ""
    echo "Warning: Waybar config not found at $WAYBAR_CONFIG"
    echo "You may need to manually configure Waybar to use: $APPIMAGE"
fi

# Opciono: kreiraj desktop entry za AppImage
DESKTOP_DIR="$HOME/.local/share/applications"
mkdir -p "$DESKTOP_DIR"

cat > "$DESKTOP_DIR/ppkiller.desktop" << EOF
[Desktop Entry]
Name=PP Killer
Comment=Port and Process Manager
Exec=$APPIMAGE
Icon=network-transmit-receive-symbolic
Type=Application
Categories=Development;System;
Terminal=false
Keywords=port;kill;process;network;
EOF

echo "Desktop entry created at $DESKTOP_DIR/ppkiller.desktop"

echo ""
echo "--------------------------------------------------"
echo "Waybar configuration updated for PP Killer AppImage!"
echo ""
echo "AppImage location: $APPIMAGE"
echo ""
echo "You can run AppImage directly:"
echo "  $APPIMAGE           (launch GUI)"
echo "  $APPIMAGE waybar   (for Waybar module)"
echo "  $APPIMAGE menu     (for Rofi menu)"
echo ""
echo "Note: AppImage is portable - keep it wherever you want!"
echo "If you move it, run this script again to update Waybar config."
echo "--------------------------------------------------"
