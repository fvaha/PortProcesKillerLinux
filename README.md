# PortKiller Pro (Linux Edition)

A premium, visually stunning port management tool for Linux, inspired by the original macOS PortKiller. Built with **Tauri**, **Rust**, and a custom **Liquid Glass** UI.

![PortKiller Preview](https://vaha.net/images/portkiller_preview.png) *(Replace with actual image link)*

## ✨ Features

- **Liquid Glass UI**: Authentic frosted glass effect with adjustable transparency and blur.
- **Responsive Design**: Adapts perfectly to any window size, from compact to full-screen.
- **Contextual Filtering**: Quickly sort ports by categories (Web, Database, System).
- **One-Click Waybar Integration**: Automatically install the PortKiller module into your Waybar config.
- **Rofi Integration**: A dedicated, styled Rofi menu for quick access from your Desktop Environment.
- **High Performance**: Powered by Rust/Tauri for minimal resource footprint.

## 🚀 Installation

### Using the Installer (Recommended)

Clone the repository and run the installation script:

```bash
git clone https://github.com/vaha/PortKillerLinux.git
cd PortKillerLinux
./install.sh
```

### Manual Installation

1. Install dependencies: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`.
2. Build with Tauri: `npm install && npx tauri build`.
3. Copy the binary from `src-tauri/target/release/portkiller` to your `~/.local/bin/`.

## 🛠️ Usage

- **GUI**: Run `portkiller` or find it in your App Launcher.
- **Waybar**: Add `custom/portkiller` to your Waybar modules. The GUI settings include an auto-installer for this!
- **Rofi Menu**: Run `portkiller menu`.

## 👨‍💻 Created By

**Vahid E.**

- Website: [vaha.net](https://vaha.net)
- Inspired by [productdevbook/port-killer](https://github.com/productdevbook/port-killer)

---
*Developed with ❤️ for the Linux community.*
