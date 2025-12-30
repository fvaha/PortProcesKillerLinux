#!/bin/bash

# PortKiller Linux - Remove Old Installation
# Uklanja staru binarnu verziju i ažurira Waybar konfiguraciju

set -e

echo "Removing old PortKiller installation..."

# Obriši staru binarnu verziju
OLD_BINARY="$HOME/.local/bin/portkiller"
if [ -f "$OLD_BINARY" ]; then
    echo "Removing old binary: $OLD_BINARY"
    rm "$OLD_BINARY"
    echo "  ✓ Old binary removed"
else
    echo "  - No old binary found at $OLD_BINARY"
fi

# Obriši stari desktop entry
DESKTOP_DIR="$HOME/.local/share/applications"
OLD_DESKTOP="$DESKTOP_DIR/portkiller.desktop"
if [ -f "$OLD_DESKTOP" ]; then
    echo "Removing old desktop entry: $OLD_DESKTOP"
    rm "$OLD_DESKTOP"
    echo "  ✓ Old desktop entry removed"
else
    echo "  - No old desktop entry found"
fi

# Ažuriraj Waybar konfiguraciju - ukloni stare reference
WAYBAR_CONFIG="$HOME/.config/waybar/config"
if [ -f "$WAYBAR_CONFIG" ]; then
    echo ""
    echo "Updating Waybar configuration..."
    
    # Backup originalne konfiguracije
    BACKUP_FILE="$WAYBAR_CONFIG.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$WAYBAR_CONFIG" "$BACKUP_FILE"
    echo "  ✓ Backup created: $BACKUP_FILE"
    
    # Ukloni stare reference na portkiller (ali zadrži custom/ppkiller ako postoji)
    # Zameni stare putanje sa placeholder-om koji će biti zamenjen kada se instalira AppImage
    sed -i "s|~/.local/bin/portkiller|REPLACE_WITH_APPIMAGE|g" "$WAYBAR_CONFIG"
    sed -i "s|$HOME/.local/bin/portkiller|REPLACE_WITH_APPIMAGE|g" "$WAYBAR_CONFIG"
    sed -i "s|/home/vaha/.local/bin/portkiller|REPLACE_WITH_APPIMAGE|g" "$WAYBAR_CONFIG"
    
    # Ažuriraj modul naziv ako je potrebno (custom/portkiller -> custom/ppkiller)
    if grep -q '"custom/portkiller"' "$WAYBAR_CONFIG" || grep -q 'custom/portkiller' "$WAYBAR_CONFIG"; then
        echo "  ✓ Updating module name: custom/portkiller -> custom/ppkiller"
        sed -i 's|"custom/portkiller"|"custom/ppkiller"|g' "$WAYBAR_CONFIG"
        sed -i "s|'custom/portkiller'|'custom/ppkiller'|g" "$WAYBAR_CONFIG"
        sed -i 's|custom/portkiller|custom/ppkiller|g' "$WAYBAR_CONFIG"
        
        # Ažuriraj i definiciju modula
        if grep -q '"custom/portkiller":' "$WAYBAR_CONFIG"; then
            sed -i 's|"custom/portkiller":|"custom/ppkiller":|g' "$WAYBAR_CONFIG"
        fi
    fi
    
    # Ako postoji REPLACE_WITH_APPIMAGE placeholder, upozori korisnika
    if grep -q "REPLACE_WITH_APPIMAGE" "$WAYBAR_CONFIG"; then
        echo ""
        echo "  ⚠ Warning: Waybar config contains 'REPLACE_WITH_APPIMAGE' placeholder"
        echo "    Run install_appimage.sh to set the correct AppImage path"
    fi
    
    echo "  ✓ Waybar configuration updated"
else
    echo "  - Waybar config not found at $WAYBAR_CONFIG"
fi

# Ažuriraj CSS fajl ako postoji
WAYBAR_CSS="$HOME/.config/waybar/style.css"
if [ -f "$WAYBAR_CSS" ]; then
    echo ""
    echo "Updating Waybar CSS..."
    
    # Backup CSS fajla
    CSS_BACKUP="$WAYBAR_CSS.backup.$(date +%Y%m%d_%H%M%S)"
    cp "$WAYBAR_CSS" "$CSS_BACKUP"
    echo "  ✓ CSS backup created: $CSS_BACKUP"
    
    # Zameni #custom-portkiller sa #custom-ppkiller
    if grep -q "#custom-portkiller" "$WAYBAR_CSS"; then
        sed -i 's|#custom-portkiller|#custom-ppkiller|g' "$WAYBAR_CSS"
        echo "  ✓ CSS updated: #custom-portkiller -> #custom-ppkiller"
    else
        echo "  - No #custom-portkiller styles found"
    fi
else
    echo "  - Waybar CSS not found at $WAYBAR_CSS"
fi

# Obriši stare backup fajlove (opciono - zadrži poslednje 5)
echo ""
echo "Cleaning up old backup files..."
BACKUP_COUNT=$(ls -1 "$HOME/.config/waybar/config.backup."* 2>/dev/null | wc -l)
if [ "$BACKUP_COUNT" -gt 5 ]; then
    ls -1t "$HOME/.config/waybar/config.backup."* 2>/dev/null | tail -n +6 | xargs rm -f
    echo "  ✓ Removed old backup files (kept last 5)"
else
    echo "  - No old backups to clean (found $BACKUP_COUNT backups)"
fi

CSS_BACKUP_COUNT=$(ls -1 "$HOME/.config/waybar/style.css.backup."* 2>/dev/null | wc -l)
if [ "$CSS_BACKUP_COUNT" -gt 5 ]; then
    ls -1t "$HOME/.config/waybar/style.css.backup."* 2>/dev/null | tail -n +6 | xargs rm -f
    echo "  ✓ Removed old CSS backups (kept last 5)"
fi

echo ""
echo "--------------------------------------------------"
echo "Old installation removed successfully!"
echo ""
echo "Next steps:"
echo "  1. Run: ./HELPERS/install_appimage.sh [path-to-AppImage]"
echo "     or use 'Install for Waybar' button in PP Killer settings"
echo ""
echo "  2. Restart Waybar:"
echo "     killall -SIGUSR2 waybar"
echo "--------------------------------------------------"
