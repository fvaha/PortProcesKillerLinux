use serde::{Deserialize, Serialize};
use std::process::Command;
use regex::Regex;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortInfo {
    pub port: String,
    pub pid: Option<i32>,
    pub process_name: Option<String>,
    pub user: String,
}

pub fn _get_ports() -> Vec<PortInfo> {
    let output = Command::new("ss")
        .args(["-nltp"])
        .output()
        .expect("failed to execute ss");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut ports = Vec::new();

    let re_users = Regex::new(r#"users:\(\("([^"]+)",pid=(\d+),"#).unwrap();
    let current_user = std::env::var("USER").unwrap_or_else(|_| "user".to_string());

    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 { continue; }

        let local_addr = parts[3];
        let port = local_addr.split(':').last().unwrap_or("").to_string();
        if port.is_empty() { continue; }

        let mut process_name = None;
        let mut pid = None;

        if let Some(caps) = re_users.captures(line) {
            let mut name = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
            pid = caps.get(2).map(|m| m.as_str().parse::<i32>().ok()).flatten();

            if name.contains('/') {
                name = name.split('/').last().unwrap_or(&name).to_string();
            }
            process_name = Some(name);
        }

        ports.push(PortInfo {
            port,
            pid,
            process_name,
            user: current_user.clone(),
        });
    }

    ports.retain(|p| p.pid.is_some());
    ports.sort_by_key(|p| p.port.parse::<u32>().unwrap_or(0));
    ports.dedup_by(|a, b| a.port == b.port);
    
    ports
}

#[tauri::command]
fn get_ports() -> Vec<PortInfo> {
    _get_ports()
}

pub fn _kill_port(pid: i32) -> bool {
    let _ = Command::new("kill").arg(pid.to_string()).status();
    let status = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .status();
    
    match status {
        Ok(s) => s.success(),
        Err(_) => false,
    }
}

#[tauri::command]
fn kill_port(pid: i32) -> bool {
    _kill_port(pid)
}

#[tauri::command]
fn open_terminal() {
    let terms = ["gnome-terminal", "konsole", "xfce4-terminal", "alacritty", "kitty", "foot", "tilix", "termite", "xterm"];
    for term in terms {
        if Command::new(term).spawn().is_ok() {
            break;
        }
    }
}

#[tauri::command]
fn setup_waybar() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "Could not find HOME dir")?;
    let config_dir = PathBuf::from(&home).join(".config/waybar");
    
    if !config_dir.exists() {
        return Err("Waybar config directory not found at ~/.config/waybar".to_string());
    }

    // 1. Update Config (simple append for definition)
    let config_path = config_dir.join("config");
    let mut config = fs::read_to_string(&config_path).map_err(|_| "Could not read waybar config")?;
    
    if !config.contains("\"custom/portkiller\"") {
        let module_def = format!(r#"
    "custom/portkiller": {{
        "format": "{{}}",
        "exec": "{}/.local/bin/portkiller waybar",
        "return-type": "json",
        "on-click": "{}/.local/bin/portkiller menu",
        "on-click-right": "{}/.local/bin/portkiller",
        "interval": 5,
        "tooltip": true
    }},"#, home, home, home);

        // Try to inject before the first closing brace of the main object
        if let Some(pos) = config.find('}') {
            config.insert_str(pos + 1, &module_def);
        } else {
            return Err("Waybar config format is invalid".to_string());
        }
        fs::write(&config_path, config).map_err(|_| "Failed to write waybar config")?;
    }

    // 2. Update CSS
    let css_path = config_dir.join("style.css");
    let mut css = fs::read_to_string(&css_path).unwrap_or_default();
    if !css.contains("#custom-portkiller") {
        let css_append = r#"
#custom-portkiller {
    background: rgba(59, 130, 246, 0.1);
    color: #60a5fa;
    border-radius: 8px;
    padding: 0 10px;
    margin: 4px 2px;
}
#custom-portkiller.active {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
}
"#;
        css.push_str(css_append);
        fs::write(&css_path, css).map_err(|_| "Failed to write style.css")?;
    }

    // 3. Reload Waybar
    let _ = Command::new("killall").arg("-SIGUSR2").arg("waybar").status();

    Ok("Waybar integrated! Please manually add 'custom/portkiller' to your 'modules-right' or 'modules-left' in waybar config.".to_string())
}

pub fn kill_all_ports() {
    let ports = _get_ports();
    for p in ports {
        if let Some(pid) = p.pid {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
        }
    }
}

pub fn get_ports_list() -> Vec<PortInfo> {
    _get_ports()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![get_ports, kill_port, open_terminal, setup_waybar])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
